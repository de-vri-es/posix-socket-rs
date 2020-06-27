//! Thin wrapper around POSIX sockets.
//!
//! The standard library sockets are nice for dealing with TCP, UDP and Unix streaming and datagram sockets.
//! However, for all other sockets, you will get no help from the standard library.
//!
//! Additionally, the standard library sockets don't always expose all underlying features of the sockets.
//! For example, you can not send file descriptors over the standard library sockets without using libc.
//!
//! This library intends to expose the POSIX socket API to Rust without cutting features.
//! It is currently still a work in progress.

use filedesc::FileDesc;
use std::io::{IoSlice, IoSliceMut};
use std::os::unix::io::{RawFd, AsRawFd, IntoRawFd, FromRawFd};
use std::os::raw::{c_int, c_void};

mod address;
pub use address::*;

/// A POSIX socket.
pub struct Socket {
	fd: FileDesc,
}

#[cfg(not(target_os = "apple"))]
const EXTRA_MSG_FLAGS: c_int = libc::MSG_NOSIGNAL;
#[cfg(target_os = "apple")]
const EXTRA_MSG_FLAGS: c_int = 0;

impl Socket {
	/// Wrap a file descriptor in a Socket.
	///
	/// On Apple systems, this sets the SO_NOSIGPIPE option to prevent SIGPIPE signals.
	fn wrap(fd: FileDesc) -> std::io::Result<Self> {
		#[cfg(target_os = "apple")]
		set_socket_option(fd, libc::SOL_SOCKET, libc::SO_NOSIGPIPE, 1 as c_int)?;
		Ok(Self { fd })
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
		socketpair(domain, kind, protocol)
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

	/// Put the socket in blocking or non-blocking mode.
	pub fn set_nonblocking(&self, non_blocking: bool) -> std::io::Result<()> {
		set_socket_option(self, libc::SOL_SOCKET, libc::O_NONBLOCK, bool_to_c_int(non_blocking))
	}

	/// Check if the socket in blocking or non-blocking mode.
	pub fn get_nonblocking(&self) -> std::io::Result<bool> {
		let raw: c_int = get_socket_option(self, libc::SOL_SOCKET, libc::O_NONBLOCK)?;
		Ok(raw != 0)
	}

	/// Gets the value of the SO_ERROR option on this socket.
	///
	/// This will retrieve the stored error in the underlying socket, clearing the field in the process.
	/// This can be useful for checking errors between calls.
	pub fn take_error(&self) -> std::io::Result<Option<std::io::Error>> {
		let raw: c_int = get_socket_option(self, libc::SOL_SOCKET, libc::SO_ERROR)?;
		if raw == 0 {
			Ok(None)
		} else {
			Ok(Some(std::io::Error::from_raw_os_error(raw)))
		}
	}

	/// Send a message over the socket to the connected peer.
	pub fn send_msg(&self, data: &[IoSlice], cdata: Option<&[u8]>, flags: c_int) -> std::io::Result<usize> {
		unsafe {
			let mut header = std::mem::zeroed::<libc::msghdr>();
			header.msg_iov = data.as_ptr() as *mut libc::iovec;
			header.msg_iovlen = data.len();
			header.msg_control = cdata.map(|x| x.as_ptr()).unwrap_or(std::ptr::null()) as *mut c_void;
			header.msg_controllen = cdata.map(|x| x.len()).unwrap_or(0);

			let ret = check_ret_isize(libc::sendmsg(self.as_raw_fd(), &header, flags | EXTRA_MSG_FLAGS))?;
			Ok(ret as usize)
		}
	}

	/// Send a message over the socket to the specified address.
	///
	/// This is only valid for connection-less protocols such as UDP or unix datagram sockets.
	pub fn send_msg_to<Address: AsSocketAddress>(&self, address: &Address, data: &[IoSlice], cdata: Option<&[u8]>, flags: c_int) -> std::io::Result<usize> {
		unsafe {
			let mut header = std::mem::zeroed::<libc::msghdr>();
			header.msg_name = address.as_sockaddr() as *mut c_void;
			header.msg_namelen = address.len();
			header.msg_iov = data.as_ptr() as *mut libc::iovec;
			header.msg_iovlen = data.len();
			header.msg_control = cdata.map(|x| x.as_ptr()).unwrap_or(std::ptr::null()) as *mut c_void;
			header.msg_controllen = cdata.map(|x| x.len()).unwrap_or(0);

			let ret = check_ret_isize(libc::sendmsg(self.as_raw_fd(), &header, flags | EXTRA_MSG_FLAGS))?;
			Ok(ret as usize)
		}
	}

	/// Receive a message on the socket from the connected peer.
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

			let ret = check_ret_isize(libc::recvmsg(self.as_raw_fd(), &mut header, flags | EXTRA_MSG_FLAGS))?;
			Ok((ret as usize, header.msg_flags))
		}
	}

	/// Receive a message on the socket from any address.
	///
	/// The address of the sender is given in the return value.
	///
	/// This is only valid for connection-less protocols such as UDP or unix datagram sockets.
	pub fn recv_msg_from<Address: AsSocketAddress>(&self, data: &[IoSliceMut], cdata: Option<&mut [u8]>, flags: c_int) -> std::io::Result<(Address, usize, c_int)> {
		let (cdata_buf, cdata_len) = if let Some(cdata) = cdata {
			(cdata.as_mut_ptr(), cdata.len())
		} else {
			(std::ptr::null_mut(), 0)
		};

		unsafe {
			let mut address : Address = std::mem::zeroed();
			let mut header = std::mem::zeroed::<libc::msghdr>();
			header.msg_name = address.as_sockaddr_mut() as *mut c_void;
			header.msg_namelen = address.max_len();
			header.msg_iov = data.as_ptr() as *mut libc::iovec;
			header.msg_iovlen = data.len();
			header.msg_control = cdata_buf as *mut c_void;
			header.msg_controllen = cdata_len;

			let ret = check_ret_isize(libc::recvmsg(self.as_raw_fd(), &mut header, flags | EXTRA_MSG_FLAGS))?;
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

fn set_socket_option<T: Copy>(fd: impl AsRawFd, level: c_int, option: c_int, value: T) -> std::io::Result<()> {
	unsafe {
		check_ret(libc::setsockopt(fd.as_raw_fd(), level, option, &value as *const T as *const c_void, std::mem::size_of::<T>() as libc::socklen_t))?;
		Ok(())
	}
}

fn get_socket_option<T: Copy>(fd: impl AsRawFd, level: c_int, option: c_int) -> std::io::Result<T> {
	unsafe {
		let mut output = std::mem::MaybeUninit::zeroed();
		let mut length = std::mem::size_of::<T>() as libc::socklen_t;
		check_ret(libc::getsockopt(fd.as_raw_fd(), level, option, output.as_mut_ptr() as *mut c_void, (&mut length) as *mut libc::socklen_t))?;
		assert_eq!(length, std::mem::size_of::<T>() as libc::socklen_t);
		Ok(output.assume_init())
	}
}

fn bool_to_c_int(value: bool) -> c_int {
	if value {
		1
	} else {
		0
	}
}
