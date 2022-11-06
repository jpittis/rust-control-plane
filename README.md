This project provides libraries for implementing Envoy control-planes in Rust.

- `data-plane-api/` (published to crates.io as
  [data-plane-api](https://crates.io/crates/data-plane-api)) provides prost and tonic
  generated protobuf and gRPC implementations of Envoy's
  [data-plane-api](https://github.com/envoyproxy/data-plane-api).
- `rust-control-plane/` provides higher-level abstractions over an xDS gRPC server
  (similar to, and modeled after
  [go-control-plane](https://github.com/envoyproxy/go-control-plane)).
- `test-harness` provides integration tests.
