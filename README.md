# posix-socket [![docs][docs-badge]][docs] [![tests][tests-badge]][tests]
[docs]: https://docs.rs/posix-socket/
[tests]: https://github.com/de-vri-es/posix-socket-rs/actions?query=workflow%3Atests
[docs-badge]: https://docs.rs/posix-socket/badge.svg
[tests-badge]: https://github.com/de-vri-es/posix-socket-rs/workflows/tests/badge.svg

Thin wrapper around POSIX sockets.

The standard library sockets are nice for dealing with TCP, UDP and Unix streaming and datagram sockets.
However, for all other sockets, you will get no help from the standard library.

Additionally, the standard library sockets don't always expose all underlying features of the sockets.
For example, you can not send file descriptors over the standard library sockets without using libc.

This library intends to expose the POSIX socket API to Rust without cutting features.
It is currently still a work in progress.
