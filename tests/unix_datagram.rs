use assert2::assert;
use posix_socket::UnixSocket;

#[test]
fn test_socketpair() {
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
