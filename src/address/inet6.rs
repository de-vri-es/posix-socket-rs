/// IPv6 socket address.
///
/// This includes an IPv6 address and a 16-bit port number.
#[derive(Clone)]
#[repr(C)]
pub struct Inet6SocketAddress {
	/// The inner C-compatible socket address.
	inner: libc::sockaddr_in6,
}

impl Inet6SocketAddress {
	/// Create an IPv6 socket address.
	pub fn new(ip: std::net::Ipv6Addr, port: u16, flowinfo: u32, scope_id: u32) -> Self {
		let inner = libc::sockaddr_in6 {
			sin6_family: libc::AF_INET6 as libc::sa_family_t,
			sin6_addr: libc::in6_addr { s6_addr: ip.octets() },
			sin6_port: port.to_be(),
			sin6_flowinfo: flowinfo,
			sin6_scope_id: scope_id,
		};
		Self::from_raw(inner)
	}

	/// Create an IPv6 socket address from a [`libc::sockaddr_in6`].
	pub fn from_raw(inner: libc::sockaddr_in6) -> Self {
		Self { inner }
	}

	/// Convert the [`SocketAddress`] into raw [`libc`] parts.
	pub fn into_raw(self) -> libc::sockaddr_in6 {
		self.inner
	}

	/// Get the IP address associated with the socket address.
	pub fn ip(&self) -> std::net::Ipv6Addr {
		self.inner.sin6_addr.s6_addr.into()
	}

	/// Set the IP address associated with the socket address.
	pub fn set_ip(&mut self, ip: std::net::Ipv6Addr) {
		self.inner.sin6_addr.s6_addr = ip.octets();
	}

	/// Get the port number associated with the socket address.
	pub fn port(&self) -> u16 {
		u16::from_be(self.inner.sin6_port)
	}

	/// Set the port number associated with the socket address.
	pub fn set_port(&mut self, port: u16) {
		self.inner.sin6_port = port.to_be();
	}

	/// Get the flow information associated with the socket address.
	fn flowinfo(&self) -> u32 {
		self.inner.sin6_flowinfo
	}

	/// Set the flow information associated with the socket address.
	pub fn set_flowinfo(&mut self, flowinfo: u32) {
		self.inner.sin6_flowinfo = flowinfo;
	}

	/// Get the scope ID associated with the socket address.
	fn scope_id(&self) -> u32 {
		self.inner.sin6_scope_id
	}

	/// Set the scope ID associated with the socket address.
	pub fn set_scope_id(&mut self, scope_id: u32) {
		self.inner.sin6_scope_id = scope_id;
	}
}

impl crate::AsSocketAddress for Inet6SocketAddress {
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

impl From<Inet6SocketAddress> for crate::SocketAddress {
	fn from(other: Inet6SocketAddress) -> Self {
		Self::from(&other)
	}
}

impl From<&Inet6SocketAddress> for crate::SocketAddress {
	fn from(other: &Inet6SocketAddress) -> Self {
		Self::from_other(other)
	}
}

impl From<std::net::SocketAddrV6> for Inet6SocketAddress {
	fn from(other: std::net::SocketAddrV6) -> Self {
		Self::from(&other)
	}
}

impl From<&std::net::SocketAddrV6> for Inet6SocketAddress {
	fn from(other: &std::net::SocketAddrV6) -> Self {
		Self::new(*other.ip(), other.port(), other.flowinfo(), other.scope_id())
	}
}

impl From<Inet6SocketAddress> for std::net::SocketAddrV6 {
	fn from(other: Inet6SocketAddress) -> Self {
		Self::from(&other)
	}
}

impl From<&Inet6SocketAddress> for std::net::SocketAddrV6 {
	fn from(other: &Inet6SocketAddress) -> Self {
		Self::new(other.ip(), other.port(), other.flowinfo(), other.scope_id())
	}
}
