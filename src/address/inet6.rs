/// IPv6 socket address.
///
/// This includes an IPv6 address and a 16-bit port number.
#[derive(Clone)]
#[repr(C)]
pub struct SocketAddressInet6 {
	/// The inner C-compatible socket address.
	inner: libc::sockaddr_in6,
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

impl crate::AsSocketAddress for SocketAddressInet6 {
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

impl From<&SocketAddressInet6> for crate::SocketAddress {
	fn from(other: &SocketAddressInet6) -> Self {
		Self::from_other(other)
	}
}
