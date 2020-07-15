use crate::{AsSocketAddress, SpecificSocketAddress};
use std::path::Path;

/// Unix socket address.
///
/// A Unix socket address can be unnamed or a filesystem path.
/// On Linux it can also be an abstract socket path, although this is not portable.
#[derive(Clone)]
#[repr(C)]
pub struct UnixSocketAddress {
	/// The inner C-compatible socket address.
	inner: libc::sockaddr_un,

	/// The length of the socket address.
	len: libc::socklen_t,
}

impl UnixSocketAddress {
	/// Create a Unix socket address from a path.
	pub fn new<P: AsRef<Path>>(path: P) -> std::io::Result<Self> {
		use std::os::unix::ffi::OsStrExt;
		let path = path.as_ref().as_os_str().as_bytes();

		unsafe {
			let mut output = Self {
				inner: libc::sockaddr_un {
					sun_family: Self::static_family(),
					sun_path: std::mem::zeroed(),
				},
				len: 0,
			};
			let path_offset = output.path_offset();
			if path.len() >= Self::max_len() as usize - path_offset - 1 {
				Err(std::io::Error::new(std::io::ErrorKind::InvalidInput, "path is too large for a socket address"))
			} else if path.is_empty() {
				Ok(output)
			} else {
				std::ptr::copy(
					path.as_ptr(),
					output.inner.sun_path.as_mut_ptr() as *mut u8,
					path.len(),
				);
				output.len = (path_offset + path.len() + 1) as libc::socklen_t;
				Ok(output)
			}
		}
	}

	/// Create a new unnamed unix socket address.
	pub fn new_unnamed() -> Self {
		unsafe {
			let mut address = Self {
				inner: libc::sockaddr_un {
					sun_family: Self::static_family(),
					sun_path: std::mem::zeroed(),
				},
				len: 0,
			};
			address.len = address.path_offset() as libc::socklen_t;
			address
		}
	}

	/// Create a Unix socket address from a [`libc::sockaddr_un`] and a length.
	pub fn from_raw(inner: libc::sockaddr_un, len: libc::socklen_t) -> Self {
		Self { inner, len }
	}

	/// Convert the [`SocketAddress`] into raw [`libc`] parts.
	pub fn into_raw(self) -> (libc::sockaddr_un, libc::socklen_t) {
		(self.inner, self.len)
	}

	/// Get the path associated with the socket address, if there is one.
	///
	/// Returns [`None`] if the socket address is unnamed or abstract,
	pub fn as_path(&self) -> Option<&Path> {
		unsafe {
			use std::os::unix::ffi::OsStrExt;
			let path_len = self.path_len();
			if path_len == 0 {
				None
			} else if self.inner.sun_path[0] == 0 {
				None
			} else {
				let path: &[u8] = std::mem::transmute(&self.inner.sun_path[..path_len - 1]);
				let path = std::ffi::OsStr::from_bytes(path);
				Some(Path::new(path))
			}
		}
	}

	/// Check if the address is unnamed.
	pub fn is_unnamed(&self) -> bool {
		self.path_len() == 0
	}

	/// Get the abstract path associated with the socket address.
	///
	/// Returns [`None`] if the socket address is not abstract.
	///
	/// Abstract Unix socket addresses are a non-portable Linux extension.
	pub fn as_abstract(&self) -> Option<&std::ffi::CStr> {
		unsafe {
			let path_len = self.path_len();
			if path_len > 0 && self.inner.sun_path[0] == 0 {
				Some(std::mem::transmute(&self.inner.sun_path[1..path_len]))
			} else {
				None
			}
		}
	}

	/// Get the offset of the path within the [`libc::sockaddr_un`] struct.
	fn path_offset(&self) -> usize {
		let start = &self.inner as *const _ as usize;
		let sun_path = &self.inner.sun_path as *const _ as usize;
		sun_path - start
	}

	/// Get the length of the path portion of the address including the terminating null byte.
	fn path_len(&self) -> usize {
		self.len() as usize - self.path_offset()
	}
}

impl SpecificSocketAddress for UnixSocketAddress {
	fn static_family() -> libc::sa_family_t {
		libc::AF_LOCAL as libc::sa_family_t
	}
}

unsafe impl AsSocketAddress for UnixSocketAddress {
	fn as_sockaddr(&self) -> *const libc::sockaddr {
		&self.inner as *const _ as *const _
	}

	fn as_sockaddr_mut(address: &mut std::mem::MaybeUninit<Self>) -> *mut libc::sockaddr {
		unsafe { &mut address.as_mut_ptr().as_mut().unwrap().inner as *mut _ as *mut _ }
	}

	fn len(&self) -> libc::socklen_t {
		self.len
	}

	fn finalize(address: std::mem::MaybeUninit<Self>, len: libc::socklen_t) -> std::io::Result<Self> {
		unsafe {
			let mut address = address.assume_init();
			if address.family() != Self::static_family() {
				return Err(std::io::Error::new(std::io::ErrorKind::InvalidData, "wrong address family, expeced AF_LOCAL"));
			}
			if len > Self::max_len() {
				return Err(std::io::Error::new(std::io::ErrorKind::InvalidData, "address too large"));
			}
			address.len = len;
			Ok(address)
		}
	}

	fn max_len() -> libc::socklen_t {
		std::mem::size_of::<libc::sockaddr_un>() as libc::socklen_t
	}
}

impl From<UnixSocketAddress> for crate::SocketAddress {
	fn from(other: UnixSocketAddress) -> Self {
		Self::from(&other)
	}
}

impl From<&UnixSocketAddress> for crate::SocketAddress {
	fn from(other: &UnixSocketAddress) -> Self {
		Self::from_other(other)
	}
}

impl From<std::os::unix::net::SocketAddr> for UnixSocketAddress {
	fn from(other: std::os::unix::net::SocketAddr) -> Self {
		Self::from(&other)
	}
}

impl From<&std::os::unix::net::SocketAddr> for UnixSocketAddress {
	fn from(other: &std::os::unix::net::SocketAddr) -> Self {
		if let Some(path) = other.as_pathname() {
			Self::new(path).unwrap()
		} else if other.is_unnamed() {
			Self::new_unnamed()
		} else {
			panic!("attempted to convert an std::unix::net::SocketAddr that is not a path and not unnamed");
		}
	}
}
