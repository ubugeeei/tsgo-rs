# Browser Binding

The browser binding is implemented as a remote client for a Rust-backed
`corsa-bind` host.

Browser runtimes cannot spawn the local `typescript-go` executable directly, so
this binding exposes an async fetch transport and a `BrowserTsgoApiClient`
facade under [`ts/client.ts`](./ts/client.ts). The expected server contract is a
single JSON endpoint that accepts `{ method, params }` and returns either:

- `{ ok: true, result }` for JSON responses
- `{ ok: true, bytesBase64 }` for binary responses
- `{ ok: false, error }` for failures
