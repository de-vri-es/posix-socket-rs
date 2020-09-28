use assert2::assert;
use posix_socket::{UnixSocket, UnixSocketAddress};
use posix_socket::ancillary::SocketAncillary;
use std::io::{IoSlice, IoSliceMut};

mod util;

#[test]
fn test_socketpair_send_recv() {
	let (a, b) = UnixSocket::pair(libc::SOCK_DGRAM, 0).unwrap();
	assert!(a.local_addr().unwrap().is_unnamed());
	assert!(a.peer_addr().unwrap().is_unnamed());
	assert!(b.local_addr().unwrap().is_unnamed());
	assert!(b.peer_addr().unwrap().is_unnamed());

	a.send(b"hello!", 0).unwrap();

	let mut buffer = [0u8; 16];
	let len = b.recv(&mut buffer, 0).unwrap();
	assert!(&buffer[..len] == b"hello!");

	drop(b);
	assert!(let Err(_) = a.send(b"goodbye!", 0));
}

#[test]
fn test_send_msg_recv_msg() {
	let (a, b) = UnixSocket::pair(libc::SOCK_DGRAM, 0).unwrap();
	assert!(let Ok(6) = a.send_msg(&[IoSlice::new(b"hello!")], None, 0));

	let mut buffer = [0u8; 16];
	let mut ancillary = SocketAncillary::new(&mut []);
	let (len, _flags) = b.recv_msg(&[IoSliceMut::new(&mut buffer)], &mut ancillary, 0).unwrap();
	assert!(len == 6);
	assert!(ancillary.len() == 0);
	assert!(ancillary.truncated() == false);
	assert!(&buffer[..len] == b"hello!");

	drop(b);
	assert!(let Err(_) = a.send_msg(&[IoSlice::new(b"goodbye!")], None, 0));
}

#[test]
fn test_unconnected_named_sockets() {
	let tempdir = util::TempDir::new().unwrap();
	let path_a = tempdir.path().join("a.sock");
	let path_b = tempdir.path().join("b.sock");
	let path_a = path_a.as_path();
	let path_b = path_b.as_path();

	let address_a = UnixSocketAddress::new(&path_a).unwrap();
	let address_b = UnixSocketAddress::new(&path_b).unwrap();
	eprintln!("binding socket_a to {}", path_a.display());
	eprintln!("binding socket_b to {}", path_b.display());

	assert!(address_a.as_path() == Some(path_a));
	assert!(address_b.as_path() == Some(path_b));

	let a = UnixSocket::new(libc::SOCK_DGRAM, 0).unwrap();
	let b = UnixSocket::new(libc::SOCK_DGRAM, 0).unwrap();
	a.bind(&address_a).unwrap();
	b.bind(&address_b).unwrap();

	assert!(a.local_addr().unwrap().as_path() == Some(path_a));
	assert!(let Err(_) = a.peer_addr());
	assert!(b.local_addr().unwrap().as_path() == Some(path_b));
	assert!(let Err(_) = b.peer_addr());

	a.send_to(b"hello!", &address_b, 0).unwrap();

	let mut buffer = [0u8; 16];
	let (sender, len) = b.recv_from(&mut buffer, 0).unwrap();
	assert!(&buffer[..len] == b"hello!");
	assert!(sender.as_path() == Some(path_a));

	drop(b);
	assert!(let Err(_) = a.send_to(b"goodbye!", &address_b, 0));
}

#[test]
fn test_connected_named_sockets() {
	let tempdir = util::TempDir::new().unwrap();
	let path_a = tempdir.path().join("a.sock");
	let path_b = tempdir.path().join("b.sock");
	let path_a = path_a.as_path();
	let path_b = path_b.as_path();

	let address_a = UnixSocketAddress::new(&path_a).unwrap();
	let address_b = UnixSocketAddress::new(&path_b).unwrap();
	eprintln!("binding socket_a to {}", path_a.display());
	eprintln!("binding socket_b to {}", path_b.display());

	assert!(address_a.as_path() == Some(path_a));
	assert!(address_b.as_path() == Some(path_b));

	let a = UnixSocket::new(libc::SOCK_DGRAM, 0).unwrap();
	let b = UnixSocket::new(libc::SOCK_DGRAM, 0).unwrap();
	a.bind(&address_a).unwrap();
	b.bind(&address_b).unwrap();
	a.connect(&address_b).unwrap();
	b.connect(&address_a).unwrap();

	assert!(a.local_addr().unwrap().as_path() == Some(path_a));
	assert!(a.peer_addr().unwrap().as_path() == Some(path_b));
	assert!(b.local_addr().unwrap().as_path() == Some(path_b));
	assert!(b.peer_addr().unwrap().as_path() == Some(path_a));

	a.send(b"hello!", 0).unwrap();

	let mut buffer = [0u8; 16];
	let len = b.recv(&mut buffer, 0).unwrap();
	assert!(&buffer[..len] == b"hello!");

	drop(b);
	assert!(let Err(_) = a.send(b"goodbye!", 0));
}
