///! `mio` support
///
/// This module enables [`mio`] support.
/// It implements [`mio::event::Source`] for [`Socket`].

use crate::Socket;
use mio::event::Source;

impl mio::event::Source for Socket {
	fn register(&mut self, registry: &mio::Registry, token: mio::Token, interests: mio::Interest) -> mio::Result<()> {
		self.as_raw_fd().register(registry, token, interests)
	}

	fn reregister(&mut self, registry: &mio::Registry, token: mio::Token, interests: mio::Interest) -> mio::Result<()> {
		self.as_raw_fd().reregister(registry, token, interests)
	}

	fn deregister(&mut self, registry: &mio::Registry) -> mio::Result<()> {
		self.as_raw_fd().deregister(registry)
	}
}
