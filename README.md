# NOTICE

This is a fork of [embedded-msgpack](https://crates.io/crates/embedded-msgpack) tailored for its usage in WebAssembly (specifically wasmRS). When possible, changes will be upstreamed to embedded-msgpack. Ideally this crate won't need to exist,  the use cases are similar but not identical. If enough is brought into the upstream crate then this package will be deprecated.

# embedded-msgpack

[![Documentation](https://docs.rs/embedded-msgpack/badge.svg)](https://docs.rs/embedded-msgpack)
[![Crates.io](https://img.shields.io/crates/v/embedded-msgpack.svg)](https://crates.io/crates/embedded-msgpack)


MessagePack serialization implementation for Rust optimized for embedded environments

## Running tests

You can run tests for all common supported configurations via `just test` if you have [`just`](https://github.com/casey/just) installed:

```sh
just test
```

## License

Licensed under either of

- Apache License, Version 2.0, (LICENSE-APACHE or http://www.apache.org/licenses/LICENSE-2.0)
- MIT license (LICENSE-MIT or http://opensource.org/licenses/MIT)

at your option.