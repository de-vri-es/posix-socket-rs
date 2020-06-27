use std::os::raw::c_int;
use std::path::Path;

/// A type usable as socket address.
pub trait AsSocketAddress {
	/// Construct a new instance that is usable to copy an address into.
	///
	/// After construction, an address may be written into the memory pointed to by [`as_sockaddr_mut()`],
	/// limited by [`max_len()`].
	/// Afterwards, [`set_len()`] will be called with the actual address length.
	fn new_empty() -> Self;

	/// Get a pointer to the socket address.
	///
	/// In reality, this should point to a struct that is compatible with [`libc::sockaddr`],
	/// but is not [`libc::sockaddr`] itself.
	fn as_sockaddr(&self) -> *const libc::sockaddr;

	/// Get a mutable pointer to the socket address.
	///
	/// In reality, this should point to a struct that is compatible with [`libc::sockaddr`],
	/// but is not [`libc::sockaddr`] itself.
	fn as_sockaddr_mut(&mut self) -> *mut libc::sockaddr;

	/// Get the lengths of the socket address.
	///
	/// This is the length of the entire socket address, including the `sa_familly` field.
	fn len(&self) -> libc::socklen_t;

	/// Update the lengths of the address.
	///
	/// This must be the length of the entire socket address, including the `sa_familly` field.
	///
	/// It is called after the kernel wrote an address to the memory pointed at by [`as_sockaddr_mut()`](AsSocketAddress::as_sockaddr_mut).
	///
	/// # Panic
	/// This function should panic if the length is invalid for the specific address type.
	fn set_len(&mut self, len: libc::socklen_t);

	/// Get the maximum size of for the socket address.
	///
	/// This is used to tell the kernel how much it is allowed to write to the memory
	/// pointed at by [`as_sockaddr_mut()`](AsSocketAddress::as_sockaddr_mut).
	fn max_len(&self) -> libc::socklen_t;
}

/// Generic socket address, large enough to hold any valid address.
#[derive(Clone)]
#[repr(C)]
pub struct SocketAddress {
	/// The inner C-compatible socket address.
	inner: libc::sockaddr_storage,

	/// The length of the socket address.
	len: libc::socklen_t,
}

/// IPv4 socket address.
///
/// This includes an IPv4 address and a 16-bit port number.
#[derive(Clone)]
#[repr(C)]
pub struct SocketAddressInet4 {
	/// The inner C-compatible socket address.
	inner: libc::sockaddr_in,
}

/// IPv6 socket address.
///
/// This includes an IPv6 address and a 16-bit port number.
#[derive(Clone)]
#[repr(C)]
pub struct SocketAddressInet6 {
	/// The inner C-compatible socket address.
	inner: libc::sockaddr_in6,
}

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

impl SocketAddress {
	/// Create a [`SocketAddress`] from a [`libc::sockaddr_storage`] and a length.
	pub fn from_raw(inner: libc::sockaddr_storage, len: libc::socklen_t) -> Self {
		Self { inner, len }
	}

	/// Create a generic [`SocketAddress`] by copying data from another address.
	pub fn from_other<Address: AsSocketAddress>(other: &Address) -> Self {
		unsafe {
			let mut output = Self::new_empty();
			std::ptr::copy(
				other.as_sockaddr(),
				output.as_sockaddr_mut(),
				other.len() as usize
			);
			output.set_len(other.len());
			output
		}
	}

	/// Convert the [`SocketAddress`] into raw [`libc`] parts.
	pub fn into_raw(self) -> (libc::sockaddr_storage, libc::socklen_t) {
		(self.inner, self.len)
	}

	/// Get the address family.
	pub fn family(&self) -> c_int {
		self.inner.ss_family as c_int
	}

	/// Get the address as an IPv4 socket address.
	///
	/// Returns [`None`] if the address is not an IPv4 socket address.
	pub fn as_inet4(&self) -> Option<SocketAddressInet4> {
		if self.family() == libc::AF_INET {
			let addr: &libc::sockaddr_in = unsafe { std::mem::transmute(&self.inner) };
			Some(SocketAddressInet4 { inner: addr.clone() })
		} else {
			None
		}
	}

	/// Get the address as an IPv6 socket address.
	///
	/// Returns [`None`] if the address is not an IPv6 socket address.
	pub fn as_inet6(&self) -> Option<SocketAddressInet6> {
		if self.family() == libc::AF_INET6 {
			let addr: &libc::sockaddr_in6 = unsafe { std::mem::transmute(&self.inner) };
			Some(SocketAddressInet6 { inner: addr.clone() })
		} else {
			None
		}
	}

	/// Get the address as an unix socket address.
	///
	/// Returns [`None`] if the address is not a unix socket address.
	pub fn as_unix(&self) -> Option<SocketAddressUnix> {
		if self.family() == libc::AF_LOCAL {
			let addr: &libc::sockaddr_un = unsafe { std::mem::transmute(&self.inner) };
			Some(SocketAddressUnix { inner: addr.clone(), len: self.len })
		} else {
			None
		}
	}
}

impl From<&SocketAddressInet4> for SocketAddress {
	fn from(other: &SocketAddressInet4) -> Self {
		Self::from_other(other)
	}
}

impl From<&SocketAddressInet6> for SocketAddress {
	fn from(other: &SocketAddressInet6) -> Self {
		Self::from_other(other)
	}
}

impl From<&SocketAddressUnix> for SocketAddress {
	fn from(other: &SocketAddressUnix) -> Self {
		Self::from_other(other)
	}
}

impl SocketAddressInet4 {
	/// Create an IPv4 socket address from an IP address and a port number.
	pub fn new(ip: [u8; 4], port: u16) -> Self {
		unsafe {
			let ip : u32 = std::mem::transmute(ip);
			let inner = libc::sockaddr_in {
				sin_family: libc::AF_INET as libc::sa_family_t,
				sin_addr: libc::in_addr { s_addr: ip },
				sin_port: port.to_be(),
				..std::mem::zeroed()
			};
			Self::from_raw(inner)
		}
	}

	/// Create an IPv4 socket address from a [`libc::sockaddr_in`].
	pub fn from_raw(inner: libc::sockaddr_in) -> Self {
		Self { inner }
	}

	/// Convert the [`SocketAddress`] into raw [`libc`] parts.
	pub fn into_raw(self) -> libc::sockaddr_in {
		self.inner
	}
}

impl SocketAddressInet6 {
	/// Create an IPv6 socket address from an IP address and a port number.
	pub fn new(ip: [u8; 16], port: u16) -> Self {
		unsafe {
			let inner = libc::sockaddr_in6 {
				sin6_family: libc::AF_INET6 as libc::sa_family_t,
				sin6_addr: libc::in6_addr { s6_addr: ip },
				sin6_port: port.to_be(),
				..std::mem::zeroed()
			};
			Self::from_raw(inner)
		}
	}

	/// Create an IPv6 socket address from a [`libc::sockaddr_in6`].
	pub fn from_raw(inner: libc::sockaddr_in6) -> Self {
		Self { inner }
	}

	/// Convert the [`SocketAddress`] into raw [`libc`] parts.
	pub fn into_raw(self) -> libc::sockaddr_in6 {
		self.inner
	}
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

impl AsSocketAddress for SocketAddress {
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
		std::mem::size_of_val(&self.inner) as libc::socklen_t
	}
}

impl AsSocketAddress for SocketAddressInet4 {
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
		self.max_len()
	}

	fn set_len(&mut self, len: libc::socklen_t) {
		assert_eq!(len, self.max_len())
	}

	fn max_len(&self) -> libc::socklen_t {
		std::mem::size_of_val(&self.inner) as libc::socklen_t
	}
}

impl AsSocketAddress for SocketAddressInet6 {
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
		self.max_len()
	}

	fn set_len(&mut self, len: libc::socklen_t) {
		assert_eq!(len, self.max_len())
	}

	fn max_len(&self) -> libc::socklen_t {
		std::mem::size_of_val(&self.inner) as libc::socklen_t
	}
}

impl AsSocketAddress for SocketAddressUnix {
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

// TODO: bunch of conversions to/from std types.
// TODO: implement Debug in a nice manner for the types.
