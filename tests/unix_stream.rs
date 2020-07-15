use assert2::assert;
use posix_socket::UnixSocket;

#[test]
fn test_socketpair() {
	let (a, b) = UnixSocket::pair(libc::SOCK_STREAM, 0).unwrap();
	assert!(a.local_addr().unwrap().is_unnamed());
	assert!(a.peer_addr().unwrap().is_unnamed());
	assert!(b.local_addr().unwrap().is_unnamed());
	assert!(b.peer_addr().unwrap().is_unnamed());

	// Send a single character so we can't have a partial read.
	a.send(b"a", 0).unwrap();

	let mut buffer = [0u8; 16];
	let len = b.recv(&mut buffer, 0).unwrap();
	assert!(&buffer[..len] == b"a");

	drop(b);
	assert!(let Err(_) = a.send(b"goodbye!", 0));
}
