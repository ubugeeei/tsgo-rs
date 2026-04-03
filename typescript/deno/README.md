# Deno Binding

The Deno binding is remote-first.

It reuses the shared TypeScript transport/client layer so Deno code can talk to
a Rust-backed `corsa-bind` host over `fetch`.
