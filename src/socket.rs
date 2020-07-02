use filedesc::FileDesc;
use std::io::{IoSlice, IoSliceMut};
use std::os::raw::{c_int, c_void};
use std::os::unix::io::{RawFd, AsRawFd, IntoRawFd, FromRawFd};

use crate::AsSocketAddress;

/// A POSIX socket.
pub struct Socket {
	fd: FileDesc,
}

#[cfg(not(any(target_os = "apple", target_os = "solaris")))]
mod extra_flags {
	pub const SENDMSG: std::os::raw::c_int = libc::MSG_NOSIGNAL;
	pub const RECVMSG: std::os::raw::c_int = libc::MSG_CMSG_CLOEXEC;
}

#[cfg(any(target_os = "apple", target_os = "solaris"))]
mod extra_flags {
	pub const SENDMSG: std::os::raw::c_int = 0;
	pub const RECVMSG: std::os::raw::c_int = 0;
}

impl Socket {
	/// Wrap a file descriptor in a Socket.
	///
	/// On Apple systems, this sets the SO_NOSIGPIPE option to prevent SIGPIPE signals.
	fn wrap(fd: FileDesc) -> std::io::Result<Self> {
		let wrapped = Self { fd };

		#[cfg(target_os = "apple")]
		wrapped.set_option(libc::SOL_SOCKET, libc::SO_NOSIGPIPE, 1 as c_int)?;

		Ok(wrapped)
	}

	/// Create a new socket with the specified domain, type and protocol.
	///
	/// The created socket has the `close-on-exec` flag set.
	/// The flag will be set atomically when the socket is created if the platform supports it.
	///
	/// See `man socket` for more information.
	pub fn new(domain: c_int, kind: c_int, protocol: c_int) -> std::io::Result<Self> {
		socket(domain, kind | libc::SOCK_CLOEXEC, protocol)
			.or_else(|e| {
				// Fall back to setting close-on-exec after creation if SOCK_CLOEXEC is not supported.
				if e.raw_os_error() == Some(libc::EINVAL) {
					let fd = socket(domain, kind, protocol)?;
					fd.set_close_on_exec(true)?;
					Ok(fd)
				} else {
					Err(e)
				}
			})
			.and_then(Self::wrap)
	}

	/// Create a connected pair of socket with the specified domain, type and protocol.
	///
	/// The created sockets have the `close-on-exec` flag set.
	/// The flag will be set atomically when the sockets are created if the platform supports it.
	///
	/// See `man socketpair` and `man socket` for more information.
	pub fn pair(domain: c_int, kind: c_int, protocol: c_int) -> std::io::Result<(Self, Self)> {
		socketpair(domain, kind | libc::SOCK_CLOEXEC, protocol)
			.or_else(|e| {
				// Fall back to setting close-on-exec after creation if SOCK_CLOEXEC is not supported.
				if e.raw_os_error() == Some(libc::EINVAL) {
					let (a, b) = socketpair(domain, kind, protocol)?;
					a.set_close_on_exec(true)?;
					b.set_close_on_exec(true)?;
					Ok((a, b))
				} else {
					Err(e)
				}
			})
			.and_then(|(a, b)| {
				Ok((Self::wrap(a)?, Self::wrap(b)?))
			})
	}

	/// Try to clone the socket.
	///
	/// This is implemented by duplicating the file descriptor.
	/// The returned [`Socket`] refers to the same kernel object.
	///
	/// The underlying file descriptor of the new socket will have the `close-on-exec` flag set.
	/// If the platform supports it, the flag will be set atomically when the file descriptor is duplicated.
	pub fn try_clone(&self) -> std::io::Result<Self> {
		Ok(Self { fd: self.fd.duplicate()? })
	}

	/// Wrap a raw file descriptor in a [`Socket`].
	///
	/// This function sets no flags or options on the file descriptor or socket.
	/// It is your own responsibility to make sure the close-on-exec flag is already set,
	/// and that the `SO_NOSIGPIPE` option is set on Apple platforms.
	pub unsafe fn from_raw_fd(fd: RawFd) -> Self {
		Self {
			fd: FileDesc::from_raw_fd(fd),
		}
	}

	/// Get the raw file descriptor.
	///
	/// This function does not release ownership of the underlying file descriptor.
	/// The file descriptor will still be closed when the [`FileDesc`] is dropped.
	pub fn as_raw_fd(&self) -> RawFd {
		self.fd.as_raw_fd()
	}

	/// Release and get the raw file descriptor.
	///
	/// This function releases ownership of the underlying file descriptor.
	/// The file descriptor will not be closed.
	pub fn into_raw_fd(self) -> RawFd {
		self.fd.into_raw_fd()
	}

	/// Set a socket option.
	///
	/// See `man setsockopt` for more information.
	fn set_option<T: Copy>(&self, level: c_int, option: c_int, value: T) -> std::io::Result<()> {
		unsafe {
			let value = &value as *const T as *const c_void;
			let length = std::mem::size_of::<T>() as libc::socklen_t;
			check_ret(libc::setsockopt(self.as_raw_fd(), level, option, value, length))?;
			Ok(())
		}
	}

	/// Get the value of a socket option.
	///
	/// See `man getsockopt` for more information.
	fn get_option<T: Copy>(&self, level: c_int, option: c_int) -> std::io::Result<T> {
		unsafe {
			let mut output = std::mem::MaybeUninit::zeroed();
			let output_ptr = output.as_mut_ptr() as *mut c_void;
			let mut length = std::mem::size_of::<T>() as libc::socklen_t;
			check_ret(libc::getsockopt(self.as_raw_fd(), level, option, output_ptr, &mut length))?;
			assert_eq!(length, std::mem::size_of::<T>() as libc::socklen_t);
			Ok(output.assume_init())
		}
	}

	/// Put the socket in blocking or non-blocking mode.
	pub fn set_nonblocking(&self, non_blocking: bool) -> std::io::Result<()> {
		self.set_option(libc::SOL_SOCKET, libc::O_NONBLOCK, bool_to_c_int(non_blocking))
	}

	/// Check if the socket in blocking or non-blocking mode.
	pub fn get_nonblocking(&self) -> std::io::Result<bool> {
		let raw: c_int = self.get_option(libc::SOL_SOCKET, libc::O_NONBLOCK)?;
		Ok(raw != 0)
	}

	/// Gets the value of the SO_ERROR option on this socket.
	///
	/// This will retrieve the stored error in the underlying socket, clearing the field in the process.
	/// This can be useful for checking errors between calls.
	pub fn take_error(&self) -> std::io::Result<Option<std::io::Error>> {
		let raw: c_int = self.get_option(libc::SOL_SOCKET, libc::SO_ERROR)?;
		if raw == 0 {
			Ok(None)
		} else {
			Ok(Some(std::io::Error::from_raw_os_error(raw)))
		}
	}

	/// Connect the socket to a remote address.
	///
	/// It depends on the exact socket type what it means to connect the socket.
	/// See `man connect` for more information.
	pub fn connect<Address: AsSocketAddress>(&self, address: Address) -> std::io::Result<()> {
		unsafe {
			check_ret(libc::connect(self.as_raw_fd(), address.as_sockaddr(), address.len()))?;
			Ok(())
		}
	}

	/// Bind the socket to a local address.
	///
	/// It depends on the exact socket type what it means to bind the socket.
	/// See `man connect` for more information.
	pub fn bind<Address: AsSocketAddress>(&self, address: Address) -> std::io::Result<()> {
		unsafe {
			check_ret(libc::bind(self.as_raw_fd(), address.as_sockaddr(), address.len()))?;
			Ok(())
		}
	}

	/// Put the socket in listening mode, ready to accept connections.
	///
	/// Once the socket is in listening mode,
	/// new connections can be accepted with [`accept()`](Socket::accept).
	///
	/// Not all socket types can be put into listening mode.
	/// See `man listen` for more information.
	pub fn listen(&self, backlog: c_int) -> std::io::Result<()> {
		unsafe {
			check_ret(libc::listen(self.as_raw_fd(), backlog))?;
			Ok(())
		}
	}

	/// Accept a new connection on the socket.
	///
	/// The socket must have been put in listening mode
	/// with a call to [`listen()`](Socket::listen).
	///
	/// Not all socket types can be put into listening mode or accept connections.
	/// See `man listen` for more information.
	pub fn accept<Address: AsSocketAddress>(&self) -> std::io::Result<(Self, Address)> {
		unsafe {
			let mut address = Address::new_empty();
			let mut len = address.max_len();
			let fd = check_ret(libc::accept4(self.as_raw_fd(), address.as_sockaddr_mut(), &mut len, libc::SOCK_CLOEXEC))?;
			let socket = Self::wrap(FileDesc::from_raw_fd(fd))?;
			address.set_len(len);
			Ok((socket, address))
		}
	}

	/// Send data over the socket to the connected peer.
	///
	/// Returns the number of transferred bytes, or an error.
	///
	/// See `man send` for more information.
	pub fn send(&self, data: &[u8], flags: c_int) -> std::io::Result<usize> {
		unsafe {
			let data_ptr = data.as_ptr() as *const c_void;
			let transferred = check_ret_isize(libc::send(self.as_raw_fd(), data_ptr, data.len(), flags | extra_flags::SENDMSG))?;
			Ok(transferred as usize)
		}
	}

	/// Send data over the socket to the specified address.
	///
	/// This function is only valid for connectionless protocols such as UDP or unix datagram sockets.
	///
	/// Returns the number of transferred bytes, or an error.
	///
	/// See `man sendto` for more information.
	pub fn send_to<Address: AsSocketAddress>(&self, data: &[u8], address: &Address, flags: c_int) -> std::io::Result<usize> {
		unsafe {
			let data_ptr = data.as_ptr() as *const c_void;
			let transferred = check_ret_isize(libc::sendto(
				self.as_raw_fd(),
				data_ptr,
				data.len(),
				flags | extra_flags::SENDMSG,
				address.as_sockaddr(), address.len()
			))?;
			Ok(transferred as usize)
		}
	}

	/// Send a message over the socket to the connected peer.
	///
	/// Returns the number of transferred bytes, or an error.
	///
	/// See `man sendmsg` for more information.
	pub fn send_msg(&self, data: &[IoSlice], cdata: Option<&[u8]>, flags: c_int) -> std::io::Result<usize> {
		unsafe {
			let mut header = std::mem::zeroed::<libc::msghdr>();
			header.msg_iov = data.as_ptr() as *mut libc::iovec;
			header.msg_iovlen = data.len();
			header.msg_control = cdata.map(|x| x.as_ptr()).unwrap_or(std::ptr::null()) as *mut c_void;
			header.msg_controllen = cdata.map(|x| x.len()).unwrap_or(0);

			let ret = check_ret_isize(libc::sendmsg(self.as_raw_fd(), &header, flags | extra_flags::SENDMSG))?;
			Ok(ret as usize)
		}
	}

	/// Send a message over the socket to the specified address.
	///
	/// This function is only valid for connectionless protocols such as UDP or unix datagram sockets.
	///
	/// Returns the number of transferred bytes, or an error.
	///
	/// See `man sendmsg` for more information.
	pub fn send_msg_to<Address: AsSocketAddress>(&self, address: &Address, data: &[IoSlice], cdata: Option<&[u8]>, flags: c_int) -> std::io::Result<usize> {
		unsafe {
			let mut header = std::mem::zeroed::<libc::msghdr>();
			header.msg_name = address.as_sockaddr() as *mut c_void;
			header.msg_namelen = address.len();
			header.msg_iov = data.as_ptr() as *mut libc::iovec;
			header.msg_iovlen = data.len();
			header.msg_control = cdata.map(|x| x.as_ptr()).unwrap_or(std::ptr::null()) as *mut c_void;
			header.msg_controllen = cdata.map(|x| x.len()).unwrap_or(0);

			let ret = check_ret_isize(libc::sendmsg(self.as_raw_fd(), &header, flags | extra_flags::SENDMSG))?;
			Ok(ret as usize)
		}
	}

	/// Receive a data on the socket from the connected peer.
	///
	/// Returns the number of transferred bytes, or an error.
	///
	/// See `man recvmsg` for more information.
	pub fn recv(&self, buffer: &mut [u8], flags: c_int) -> std::io::Result<usize> {
		unsafe {
			let buffer_ptr = buffer.as_mut_ptr() as *mut c_void;
			let transferred = check_ret_isize(libc::recv(self.as_raw_fd(), buffer_ptr, buffer.len(), flags | extra_flags::RECVMSG))?;
			Ok(transferred as usize)
		}
	}

	/// Receive a data on the socket.
	///
	/// Returns the address of the sender and the number of transferred bytes, or an error.
	///
	/// See `man recvmsg` for more information.
	pub fn recv_from<Address: AsSocketAddress>(&self, buffer: &mut [u8], flags: c_int) -> std::io::Result<(Address, usize)> {
		unsafe {
			let buffer_ptr = buffer.as_mut_ptr() as *mut c_void;
			let mut address = Address::new_empty();
			let mut address_len = address.max_len();
			let transferred = check_ret_isize(libc::recvfrom(
				self.as_raw_fd(),
				buffer_ptr,
				buffer.len(),
				flags,
				address.as_sockaddr_mut(),
				&mut address_len
			))?;

			address.set_len(address_len);
			Ok((address, transferred as usize))
		}
	}

	/// Receive a message on the socket from the connected peer.
	///
	/// Returns the number of transferred bytes, or an error.
	///
	/// See `man recvmsg` for more information.
	pub fn recv_msg(&self, data: &[IoSliceMut], cdata: Option<&mut [u8]>, flags: c_int) -> std::io::Result<(usize, c_int)> {
		let (cdata_buf, cdata_len) = if let Some(cdata) = cdata {
			(cdata.as_mut_ptr(), cdata.len())
		} else {
			(std::ptr::null_mut(), 0)
		};

		unsafe {
			let mut header = std::mem::zeroed::<libc::msghdr>();
			header.msg_iov = data.as_ptr() as *mut libc::iovec;
			header.msg_iovlen = data.len();
			header.msg_control = cdata_buf as *mut c_void;
			header.msg_controllen = cdata_len;

			let ret = check_ret_isize(libc::recvmsg(self.as_raw_fd(), &mut header, flags | extra_flags::RECVMSG))?;
			Ok((ret as usize, header.msg_flags))
		}
	}

	/// Receive a message on the socket from any address.
	///
	/// Returns the address of the sender and the number of transferred bytes, or an error.
	///
	/// See `man recvmsg` for more information.
	pub fn recv_msg_from<Address: AsSocketAddress>(&self, data: &[IoSliceMut], cdata: Option<&mut [u8]>, flags: c_int) -> std::io::Result<(Address, usize, c_int)> {
		let (cdata_buf, cdata_len) = if let Some(cdata) = cdata {
			(cdata.as_mut_ptr(), cdata.len())
		} else {
			(std::ptr::null_mut(), 0)
		};

		unsafe {
			let mut address = Address::new_empty();
			let mut header = std::mem::zeroed::<libc::msghdr>();
			header.msg_name = address.as_sockaddr_mut() as *mut c_void;
			header.msg_namelen = address.max_len();
			header.msg_iov = data.as_ptr() as *mut libc::iovec;
			header.msg_iovlen = data.len();
			header.msg_control = cdata_buf as *mut c_void;
			header.msg_controllen = cdata_len;

			let ret = check_ret_isize(libc::recvmsg(self.as_raw_fd(), &mut header, flags | extra_flags::RECVMSG))?;
			address.set_len(header.msg_namelen);
			Ok((address, ret as usize, header.msg_flags))
		}
	}
}

impl FromRawFd for Socket {
	unsafe fn from_raw_fd(fd: RawFd) -> Self {
		Self::from_raw_fd(fd)
	}
}

impl AsRawFd for Socket {
	fn as_raw_fd(&self) -> RawFd {
		self.as_raw_fd()
	}
}

impl AsRawFd for &'_ Socket {
	fn as_raw_fd(&self) -> RawFd {
		(*self).as_raw_fd()
	}
}

impl IntoRawFd for Socket {
	fn into_raw_fd(self) -> RawFd {
		self.into_raw_fd()
	}
}

/// Wrap the return value of a libc function in an [`std::io::Result`].
///
/// If the return value is -1, [`last_os_error()`](std::io::Error::last_os_error) is returned.
/// Otherwise, the return value is returned wrapped as [`Ok`].
fn check_ret(ret: c_int) -> std::io::Result<c_int> {
	if ret == -1 {
		Err(std::io::Error::last_os_error())
	} else {
		Ok(ret)
	}
}

/// Wrap the return value of a libc function in an [`std::io::Result`].
///
/// If the return value is -1, [`last_os_error()`](std::io::Error::last_os_error) is returned.
/// Otherwise, the return value is returned wrapped as [`Ok`].
fn check_ret_isize(ret: isize) -> std::io::Result<isize> {
	if ret == -1 {
		Err(std::io::Error::last_os_error())
	} else {
		Ok(ret)
	}
}

/// Create a socket and wrap the created file descriptor.
fn socket(domain: c_int, kind: c_int, protocol: c_int) -> std::io::Result<FileDesc> {
	unsafe {
		let fd = check_ret(libc::socket(domain, kind, protocol))?;
		Ok(FileDesc::from_raw_fd(fd))
	}
}

/// Create a socket pair and wrap the created file descriptors.
fn socketpair(domain: c_int, kind: c_int, protocol: c_int) -> std::io::Result<(FileDesc, FileDesc)> {
	unsafe {
		let mut fds = [0; 2];
		check_ret(libc::socketpair(domain, kind, protocol, fds.as_mut_ptr()))?;
		Ok((
			FileDesc::from_raw_fd(fds[0]),
			FileDesc::from_raw_fd(fds[1]),
		))
	}
}

fn bool_to_c_int(value: bool) -> c_int {
	if value {
		1
	} else {
		0
	}
}
