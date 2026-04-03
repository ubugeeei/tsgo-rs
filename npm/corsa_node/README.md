# @tsgo-rs/node

`@tsgo-rs/node` exposes the `tsgo-rs` Rust workspace to Node.js through
`napi-rs`.

## What it ships

- native Node.js bindings for the `tsgo-rs` API and LSP surface
- an ESM TypeScript wrapper under `dist/`
- no bundled `typescript-go` executable

## Runtime requirement

You must provide a compatible `typescript-go` (`tsgo`) executable yourself and
pass its path through `TsgoApiClient.spawn({ executable: "/path/to/tsgo" })`.

## Development

```bash
vp install
vp run -w build_wrapper
vp test run --config ./vite.config.ts npm/tsgo_rs_node/ts/**/*.test.ts
```

Repository-level executable examples live under [`examples/`](../../examples/README.md),
including mock-client, virtual-document, distributed-orchestrator, and
real-`tsgo` snapshot samples.
