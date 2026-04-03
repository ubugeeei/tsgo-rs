# Zig Binding

The Zig binding is implemented as a thin wrapper around the shared C ABI.

[`corsa_bind.zig`](./corsa_bind.zig) exposes:

- version and unsafe-type predicate helpers
- JSON-based API client calls
- virtual document helpers

Build the Rust C ABI first with `cargo build -p corsa_bind_c`, then link the
resulting library when compiling Zig code.
