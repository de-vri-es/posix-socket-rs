use std::path::Path;
use crate::AsSocketAddress;

/// Unix socket address.
///
/// A Unix socket address can be unnamed or a filesystem path.
/// On Linux it can also be an abstract socket path, although this is not portable.
#[derive(Clone)]
#[repr(C)]
pub struct SocketAddressUnix {
	/// The inner C-compatible socket address.
	inner: libc::sockaddr_un,

	/// The length of the socket address.
	len: libc::socklen_t,
}

impl SocketAddressUnix {
	/// Create a Unix socket address from a path.
	pub fn new<P: AsRef<Path>>(path: P) -> std::io::Result<Self> {
		use std::os::unix::ffi::OsStrExt;
		let path = path.as_ref().as_os_str().as_bytes();

		unsafe {
			let mut output = Self::new_empty();
			let path_offset = output.path_offset();
			if path.len() >= output.max_len() as usize - path_offset {
				Err(std::io::Error::new(std::io::ErrorKind::InvalidInput, "path is too large for a socket address"))
			} else if path.is_empty() {
				Ok(output)
			} else {
				std::ptr::copy(
					path.as_ptr(),
					output.as_sockaddr_mut() as *mut u8,
					path.len(),
				);
				output.set_len((path_offset + path.len() + 1) as u32);
				Ok(output)
			}
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

impl crate::AsSocketAddress for SocketAddressUnix {
	fn new_empty() -> Self {
		let mut address = Self {
			inner: unsafe { std::mem::zeroed() },
			len: 0,
		};
		address.len = address.path_offset() as libc::socklen_t;
		address
	}

	fn as_sockaddr(&self) -> *const libc::sockaddr {
		&self.inner as *const _ as *const _
	}

	fn as_sockaddr_mut(&mut self) -> *mut libc::sockaddr {
		&mut self.inner as *mut _ as *mut _
	}

	fn len(&self) -> libc::socklen_t {
		self.len
	}

	fn set_len(&mut self, len: libc::socklen_t) {
		assert!(len <= self.max_len());
		self.len = len
	}

	fn max_len(&self) -> libc::socklen_t {
		std::mem::size_of::<Self>() as libc::socklen_t
	}
}

impl From<SocketAddressUnix> for crate::SocketAddress {
	fn from(other: SocketAddressUnix) -> Self {
		Self::from(&other)
	}
}

impl From<&SocketAddressUnix> for crate::SocketAddress {
	fn from(other: &SocketAddressUnix) -> Self {
		Self::from_other(other)
	}
}

impl From<std::os::unix::net::SocketAddr> for SocketAddressUnix {
	fn from(other: std::os::unix::net::SocketAddr) -> Self {
		Self::from(&other)
	}
}

impl From<&std::os::unix::net::SocketAddr> for SocketAddressUnix {
	fn from(other: &std::os::unix::net::SocketAddr) -> Self {
		if let Some(path) = other.as_pathname() {
			Self::new(path).unwrap()
		} else if other.is_unnamed() {
			Self::new_empty()
		} else {
			panic!("attempted to convert an std::unix::net::SocketAddr that is not a path and not unnamed");
		}
	}
}
