use std::os::raw::c_int;

mod inet4;
mod inet6;
mod unix;

pub use inet4::*;
pub use inet6::*;
pub use unix::*;

// TODO: implement Debug in a nice manner for the types.

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
	pub fn as_inet4(&self) -> Option<Inet4SocketAddress> {
		if self.family() == libc::AF_INET {
			let addr: &libc::sockaddr_in = unsafe { std::mem::transmute(&self.inner) };
			Some(Inet4SocketAddress::from_raw(*addr))
		} else {
			None
		}
	}

	/// Get the address as an IPv6 socket address.
	///
	/// Returns [`None`] if the address is not an IPv6 socket address.
	pub fn as_inet6(&self) -> Option<Inet6SocketAddress> {
		if self.family() == libc::AF_INET6 {
			let addr: &libc::sockaddr_in6 = unsafe { std::mem::transmute(&self.inner) };
			Some(Inet6SocketAddress::from_raw(*addr))
		} else {
			None
		}
	}

	/// Get the address as an unix socket address.
	///
	/// Returns [`None`] if the address is not a unix socket address.
	pub fn as_unix(&self) -> Option<UnixSocketAddress> {
		if self.family() == libc::AF_LOCAL {
			let addr: &libc::sockaddr_un = unsafe { std::mem::transmute(&self.inner) };
			Some(UnixSocketAddress::from_raw(*addr, self.len))
		} else {
			None
		}
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
