//! Thin wrapper around POSIX sockets.
//!
//! The standard library sockets are nice for dealing with TCP, UDP and Unix streaming and datagram sockets.
//! However, for all other sockets, you will get no help from the standard library.
//!
//! Additionally, the standard library sockets don't always expose all underlying features of the sockets.
//! For example, you can not send file descriptors over the standard library sockets without using libc.
//!
//! This library intends to expose the POSIX socket API to Rust without cutting features.
//! It is currently still a work in progress.

mod address;
pub use address::*;

mod socket;
pub use socket::*;
