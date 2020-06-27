use std::os::raw::c_int;

pub trait AsSocketAddress {
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

/// Generic socket address.
///
/// This struct is large enough to hold any socket address.
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
/// This could be an unnamed address or a filesystem path.
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

impl AsSocketAddress for SocketAddress {
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
// TODO: bunch of conversions to/from libc types.
// TODO: implement Debug in a nice manner for the types.
