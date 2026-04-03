# Browser Binding

The browser-facing TypeScript layer lives here.

Today this surface is transport-first: browsers cannot spawn the local
`typescript-go` executable directly, so the binding reuses the shared async
client and fetch transport from [`../typescript`](../typescript/README.md).

This is also the place where a future wasm-backed browser adapter can live
without changing the public browser-facing folder name again.
