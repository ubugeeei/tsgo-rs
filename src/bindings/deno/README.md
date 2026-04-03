# Deno Binding

The Deno binding is implemented as a remote-first module.

Deno does not rely on the local Node-API addon here; instead it reuses the
browser transport/client abstraction so Deno code can talk to a Rust-backed
`corsa-bind` host over fetch.

See [`mod.ts`](./mod.ts).
