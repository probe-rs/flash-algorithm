# flash-algorithm

A crate to write CMSIS-DAP flash algorithms for flashing embedded targets.
This crate is an abstrction over https://open-cmsis-pack.github.io/Open-CMSIS-Pack-Spec/main/html/flashAlgorithm.html which takes care of proper placement of functions in the respective ELF sections and linking properly.

[![crates.io](https://img.shields.io/crates/v/flash-algorithm)](https://crates.io/crates/flash-algorithm) [![documentation](https://docs.rs/flash-algorithm/badge.svg)](https://docs.rs/flash-algorithm) [![Actions Status](https://img.shields.io/github/actions/workflow/status/probe-rs/flash-algorithm/ci.yml?branch=master)](https://github.com/probe-rs/flash-algorithm/actions) [![chat](https://img.shields.io/badge/chat-probe--rs%3Amatrix.org-brightgreen)](https://matrix.to/#/#probe-rs:matrix.org)

To write a flash algorithm, follow the instructions in https://github.com/probe-rs/flash-algorithm-template.

# License

This thingy is licensed under either of

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or
  http://www.apache.org/licenses/LICENSE-2.0)

- MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.
