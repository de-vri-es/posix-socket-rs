//! Thin wrapper around POSIX sockets.
//!
//! The standard library sockets are good for dealing with TCP, UDP and Unix streaming and datagram sockets.
//! However, for other sockets, you will get no help from the standard library.
//!
//! Additionally, the standard library sockets don't always expose all underlying features of the sockets.
//! For example, you can not send file descriptors over the standard library sockets without resorting to `libc`.
//!
//! This library intends to expose the POSIX socket API to Rust without cutting features.
//! It is currently still a work in progress.

mod address;
pub use address::*;

mod socket;
pub use socket::*;

#[cfg(fceature = "mio")]
pub mod mio;

pub type UnixSocket = Socket<UnixSocketAddress>;
pub type Inet4Socket = Socket<Inet4SocketAddress>;
pub type Inet6Socket = Socket<Inet6SocketAddress>;

/// Disable SIGPIPE for the current process.
///
/// Writing to a closed socket may cause a SIGPIPE to be sent to the process (depending on the socket type).
/// On most platforms this is prevented, either by using the `MSG_NOSIGNAL` flag when writing
/// or by setting the `SO_NOSIGPIPE` socket option.
///
/// However, if a platform does not support `MSG_NOSIGNAL` or `SO_NOGSISPIPE`,
/// the signal needs to be handled or the process will be terminated by the kernel.
/// Calling [`disable_sigpipe()`] make sure the signal is ignored without terminating the process.
pub fn disable_sigpipe() -> std::io::Result<()> {
	unsafe {
		if libc::signal(libc::SIGPIPE, libc::SIG_IGN) == libc::SIG_ERR {
			Err(std::io::Error::last_os_error())
		} else {
			Ok(())
		}
	}
}
