[![crates.io](https://img.shields.io/crates/v/hrpc)](https://crates.io/crates/hrpc) [![release docs](https://img.shields.io/docsrs/hrpc)](https://docs.rs/hrpc) [![docs](https://img.shields.io/badge/docs-master-blue)](https://harmonyapp.io/hrpc-rs)

# hrpc-rs

This repo contains an implementation of [hRPC](https://github.com/harmony-development/hrpc) in Rust:
- `crates/hrpc` contains server and client code,
- `crates/hrpc-build` is a codegen which can be used on protobuf files,
- `examples/interop` is used to test the implementation and codegen against the Go sample server / client.
- `examples` contains commented examples.
    - To run an example's server: `cargo run --package example_name --bin server`
    - To run an example's client: `cargo run --package example_name --bin client`

## Getting started

Check out the [chat](./examples/chat) example!