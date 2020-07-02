use assert2::assert;
use posix_socket::Socket;

#[test]
fn test_send() {
	let (a, b) = Socket::pair(libc::AF_LOCAL, libc::SOCK_SEQPACKET, 0).unwrap();
	a.send(b"hello!", 0).unwrap();

	let mut buffer = [0u8; 16];
	let len = b.recv(&mut buffer, 0).unwrap();
	assert!(&buffer[..len] == b"hello!");

	drop(b);
	assert!(let Err(_) = a.send(b"goodbye!", 0));
}
