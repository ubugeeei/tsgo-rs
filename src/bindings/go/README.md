# Go Binding

The Go binding is implemented as a thin cgo wrapper over the shared C ABI.

The package lives under [`corsa_bind/`](./corsa_bind/) and mirrors the C
surface with Go-friendly helpers for:

- version and unsafe-type predicates
- JSON-based API client calls
- virtual document lifecycle helpers

Build the Rust C ABI first with `cargo build -p corsa_bind_c`, then link it into
the Go build using your usual `CGO_CFLAGS` / `CGO_LDFLAGS` setup.
