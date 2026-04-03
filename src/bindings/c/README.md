# C Binding

The shared native binding layer lives in [`corsa_bind_c/`](./corsa_bind_c/).

It exposes a JSON-first C ABI over the Rust core:

- `CorsaBindApiClient` for tsgo API access
- `CorsaBindVirtualDocument` for overlay document manipulation
- unsafe-type predicates
- string and byte ownership helpers for FFI consumers
