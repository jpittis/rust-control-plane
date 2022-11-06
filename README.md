This project provides libraries for implementing Envoy control-planes in Rust.

- `data-plane-api/` (published to crates.io as
  [data-plane-api](https://crates.io/crates/data-plane-api)) provides prost and tonic
  generated protobuf and gRPC implementations of Envoy's
  [data-plane-api](https://github.com/envoyproxy/data-plane-api). Likely complete, and
  production ready.
- `rust-control-plane/` (published to creates.io as
  [rust-control-plane](https://crates.io/crates/rust-control-plane)) provides higher-level
  abstractions over an xDS gRPC server (similar to, and modeled after
  [go-control-plane](https://github.com/envoyproxy/go-control-plane)). Not complete, nor
  production ready yet.
- `test-harness` provides integration tests.
