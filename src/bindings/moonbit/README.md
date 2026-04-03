# MoonBit Binding

The MoonBit binding is implemented as a C-FFI layer over the shared C ABI.

- [`src/corsa_bind.mbt`](./src/corsa_bind.mbt) declares the MoonBit-facing
  extern types and wrappers.
- [`src/cwrap.c`](./src/cwrap.c) converts returned C strings into MoonBit-owned
  strings so the high-level MoonBit API stays string-friendly.

Build the Rust C ABI first with `cargo build -p corsa_bind_c`, then compile the
MoonBit package against the generated library and header.
