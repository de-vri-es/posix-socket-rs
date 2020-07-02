use assert2::assert;
use posix_socket::Socket;

#[test]
fn test_send() {
	let (a, b) = Socket::pair(libc::AF_LOCAL, libc::SOCK_STREAM, 0).unwrap();
	a.send(b"a", 0).unwrap();

	let mut buffer = [0u8; 16];
	let len = b.recv(&mut buffer, 0).unwrap();
	assert!(&buffer[..len] == b"a");

	drop(b);
	assert!(let Err(_) = a.send(b"goodbye!", 0));
}
