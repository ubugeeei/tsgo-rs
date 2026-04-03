# TypeScript Node.js Surface

The TypeScript-facing Node.js entrypoint lives here.

It re-exports the native `napi-rs` wrapper implemented under
[`src/bindings/nodejs/corsa_bind_node`](../../src/bindings/nodejs/corsa_bind_node)
and also exposes the shared remote transport helpers from
[`../typescript`](../typescript/README.md).
