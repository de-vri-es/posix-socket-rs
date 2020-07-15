use std::os::raw::c_int;

mod inet4;
mod inet6;
mod unix;

pub use inet4::*;
pub use inet6::*;
pub use unix::*;

// TODO: implement Debug in a nice manner for the types.

/// A socket address that supports multiple address families at runtime.
pub trait GenericSocketAddress: AsSocketAddress {}

/// A socket address that only supports one specific family.
pub trait SpecificSocketAddress: AsSocketAddress {
	/// The address family supported by this socket address.
	fn static_family() -> libc::sa_family_t;
}

/// A type that is binary compatible with a socket address.
///
/// # Safety
/// It must be valid to construct a new address as [`std::mem::MaybeUninit::new_zeroed()`]
/// and then write the socket address to the pointer returned by [`as_sockaddr_mut()`].
pub unsafe trait AsSocketAddress: Sized {
	/// Get a pointer to the socket address.
	///
	/// In reality, this should point to a struct that is compatible with [`libc::sockaddr`],
	/// but is not [`libc::sockaddr`] itself.
	fn as_sockaddr(&self) -> *const libc::sockaddr;

	/// Get the lengths of the socket address.
	///
	/// This is the length of the entire socket address, including the `sa_family` field.
	fn len(&self) -> libc::socklen_t;

	/// Get the address family of the socket address.
	fn family(&self) -> libc::sa_family_t {
		unsafe {
			(*self.as_sockaddr()).sa_family
		}
	}

	/// Get a mutable pointer to the socket address.
	///
	/// In reality, this should point to a struct that is compatible with [`libc::sockaddr`],
	/// but is not [`libc::sockaddr`] itself.
	fn as_sockaddr_mut(address: &mut std::mem::MaybeUninit<Self>) -> *mut libc::sockaddr;

	/// Get the maximum size of for the socket address.
	///
	/// This is used to tell the kernel how much it is allowed to write to the memory
	/// pointed at by [`as_sockaddr_mut()`](AsSocketAddress::as_sockaddr_mut).
	fn max_len() -> libc::socklen_t;

	/// Finalize a socket address that has been written into by the kernel.
	///
	/// This should check the address family and the length to ensure the address is valid.
	/// The length is the length of the entire socket address, including the `sa_family` field.
	fn finalize(address: std::mem::MaybeUninit<Self>, len: libc::socklen_t) -> std::io::Result<Self>;
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
			let mut output = std::mem::MaybeUninit::zeroed();
			std::ptr::copy(
				other.as_sockaddr(),
				AsSocketAddress::as_sockaddr_mut(&mut output),
				other.len() as usize
			);
			AsSocketAddress::finalize(output, other.len()).unwrap()
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

unsafe impl AsSocketAddress for SocketAddress {
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
			if len > Self::max_len() {
				return Err(std::io::Error::new(std::io::ErrorKind::InvalidData, "address too large"));
			}
			address.len = len;
			Ok(address)
		}
	}

	fn max_len() -> libc::socklen_t {
		std::mem::size_of::<libc::sockaddr_storage>() as libc::socklen_t
	}
}

impl GenericSocketAddress for SocketAddress {}
