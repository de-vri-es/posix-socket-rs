/// IPv4 socket address.
///
/// This includes an IPv4 address and a 16-bit port number.
#[derive(Clone)]
#[repr(C)]
pub struct SocketAddressInet4 {
	/// The inner C-compatible socket address.
	inner: libc::sockaddr_in,
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

impl crate::AsSocketAddress for SocketAddressInet4 {
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

impl From<&SocketAddressInet4> for crate::SocketAddress {
	fn from(other: &SocketAddressInet4) -> Self {
		Self::from_other(other)
	}
}
