use crate::SpecificSocketAddress;

/// IPv4 socket address.
///
/// This includes an IPv4 address and a 16-bit port number.
#[derive(Clone)]
#[repr(C)]
pub struct Inet4SocketAddress {
	/// The inner C-compatible socket address.
	inner: libc::sockaddr_in,
}

impl Inet4SocketAddress {
	/// Create an IPv4 socket address from an IP address and a port number.
	pub fn new(ip: &std::net::Ipv4Addr, port: u16) -> Self {
		unsafe {
			let ip : u32 = std::mem::transmute(ip.octets());
			let inner = libc::sockaddr_in {
				sin_family: Self::static_family(),
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

	/// Get the IP address associated with the socket address.
	pub fn ip(&self) -> std::net::Ipv4Addr {
		unsafe {
			let ip: [u8; 4] = std::mem::transmute(self.inner.sin_addr.s_addr);
			ip.into()
		}
	}

	/// Set the IP address associated with the socket address.
	pub fn set_ip(&mut self, ip: std::net::Ipv4Addr) {
		unsafe {
			let ip: u32 = std::mem::transmute(ip.octets());
			self.inner.sin_addr.s_addr = ip;
		}
	}

	/// Get the port number associated with the socket address.
	pub fn port(&self) -> u16 {
		u16::from_be(self.inner.sin_port)
	}

	/// Set the port number associated with the socket address.
	pub fn set_port(&mut self, port: u16) {
		self.inner.sin_port = port.to_be();
	}
}

impl SpecificSocketAddress for Inet4SocketAddress {
	fn static_family() -> libc::sa_family_t {
		libc::AF_INET as libc::sa_family_t
	}
}

unsafe impl crate::AsSocketAddress for Inet4SocketAddress {
	fn as_sockaddr(&self) -> *const libc::sockaddr {
		&self.inner as *const _ as *const _
	}

	fn as_sockaddr_mut(address: &mut std::mem::MaybeUninit<Self>) -> *mut libc::sockaddr {
		unsafe { &mut address.as_mut_ptr().as_mut().unwrap().inner as *mut _ as *mut _ }
	}

	fn len(&self) -> libc::socklen_t {
		Self::max_len()
	}

	fn finalize(address: std::mem::MaybeUninit<Self>, len: libc::socklen_t) -> std::io::Result<Self> {
		unsafe {
			let address = address.assume_init();
			if address.family() != Self::static_family() {
				return Err(std::io::Error::new(std::io::ErrorKind::InvalidData, "wrong address family, expected AF_INET"));
			}
			if len != Self::max_len() {
				return Err(std::io::Error::new(std::io::ErrorKind::InvalidData, "wrong address size"));
			}
			Ok(address)
		}
	}

	fn max_len() -> libc::socklen_t {
		std::mem::size_of::<libc::sockaddr_in>() as libc::socklen_t
	}
}

impl From<Inet4SocketAddress> for crate::SocketAddress {
	fn from(other: Inet4SocketAddress) -> Self {
		Self::from(&other)
	}
}

impl From<&Inet4SocketAddress> for crate::SocketAddress {
	fn from(other: &Inet4SocketAddress) -> Self {
		Self::from_other(other)
	}
}

impl From<std::net::SocketAddrV4> for Inet4SocketAddress {
	fn from(other: std::net::SocketAddrV4) -> Self {
		Self::from(&other)
	}
}

impl From<&std::net::SocketAddrV4> for Inet4SocketAddress {
	fn from(other: &std::net::SocketAddrV4) -> Self {
		Self::new(other.ip(), other.port())
	}
}

impl From<Inet4SocketAddress> for std::net::SocketAddrV4 {
	fn from(other: Inet4SocketAddress) -> Self {
		Self::from(&other)
	}
}

impl From<&Inet4SocketAddress> for std::net::SocketAddrV4 {
	fn from(other: &Inet4SocketAddress) -> Self {
		Self::new(other.ip(), other.port())
	}
}
