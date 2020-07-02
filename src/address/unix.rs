use std::path::Path;

/// Unix socket address.
///
/// A Unix socket address can be unnamed or a filesystem path.
/// On Linux it can also be an abstract socket path.
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
		let path = path.as_ref().as_os_str();

		unsafe {
			let mut output = Self::new_empty();
			let path_offset = output.path_offset();
			// TODO: does self.len() include the trailing null byte?
			if path.len() >= output.max_len() as usize - path_offset {
				Err(std::io::Error::new(std::io::ErrorKind::InvalidInput, "path too large for socket address"))
			} else {
				std::ptr::copy(
					path.as_bytes().as_ptr(),
					output.as_sockaddr_mut() as *mut u8,
					path.len(),
				);
				output.set_len((path_offset + path.len()) as u32);
				Ok(output)
			}
		}
	}

	/// Create an IPv6 socket address from a [`libc::sockaddr_un`] and a length.
	pub fn from_raw(inner: libc::sockaddr_un, len: libc::socklen_t) -> Self {
		Self { inner, len }
	}

	/// Convert the [`SocketAddress`] into raw [`libc`] parts.
	pub fn into_raw(self) -> (libc::sockaddr_un, libc::socklen_t) {
		(self.inner, self.len)
	}

	/// Get the socket address as a [`Path`].
	///
	/// If the socket address is unnamed or abstract,
	/// this returns None.
	pub fn as_path(&self) -> Option<&Path> {
		unsafe {
			use std::os::unix::ffi::OsStrExt;
			let path_len = self.len() as usize - self.path_offset();
			if path_len == 0 {
				None
			} else if self.inner.sun_path[0] == 0 {
				None
			} else {
				// TODO: does self.len() include the trailing null byte?
				let path: &[u8] = std::mem::transmute(&self.inner.sun_path[..path_len - 1]);
				let path = std::ffi::OsStr::from_bytes(path);
				Some(Path::new(path))
			}
		}
	}

	/// Check if the address is unnamed.
	pub fn is_unnamed(&self) -> bool {
		let path_len = self.len() as usize - self.path_offset();
		path_len == 0
	}

	/// Get the offset of the path within the [`libc::sockaddr_un`] struct.
	fn path_offset(&self) -> usize {
		let start = &self.inner as *const _ as usize;
		let sun_path = &self.inner.sun_path as *const _ as usize;
		sun_path - start
	}
}

impl crate::AsSocketAddress for SocketAddressUnix {
	fn new_empty() -> Self {
		unsafe { std::mem::zeroed() }
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

impl From<&SocketAddressUnix> for crate::SocketAddress {
	fn from(other: &SocketAddressUnix) -> Self {
		Self::from_other(other)
	}
}
