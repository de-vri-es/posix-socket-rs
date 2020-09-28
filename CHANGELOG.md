v0.2.0:
  * Implement `connect`, `bind`, `listen` and `accept`.
  * Implement `send`, `send_to`, `recv` and `recv_from`.
  * Expose `set_option` and `get_option`.
  * Fix creation of socket pairs with close-on-exec set.
  * Add `mio` support as optional feature.
  * Add support for parsing ancillary data (copied from experimental `std` PR).

v0.1.0:
  * Initial release.
