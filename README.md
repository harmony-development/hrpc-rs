[![crates.io](https://img.shields.io/crates/v/hrpc)](https://crates.io/crates/hrpc) [![release docs](https://img.shields.io/docsrs/hrpc)](https://docs.rs/hrpc) [![docs](https://img.shields.io/badge/docs-master-blue)](https://harmonyapp.io/hrpc-rs)

# hrpc-rs

This repo contains an implementation of [hRPC](https://github.com/harmony-development/hrpc) in Rust:
- `hrpc` contains server and client code,
- `hrpc-build` is a codegen which can be used on protobuf files,
- `interop` is an example and is also used to test the implementation and codegen against the Go sample server.