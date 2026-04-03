# @corsa-bind/node

`@corsa-bind/node` exposes the `corsa-bind` Rust workspace to Node.js through
`napi-rs`.

The naming mirrors TypeScript's own internal terminology: `Corsa` is the
codename for the native TypeScript 7 effort, while `Strada` refers to the
existing JS-based line. `@corsa-bind/node` is named to signal that it wraps the
native `typescript-go` side of that ecosystem.

## What it ships

- native Node.js bindings for the `corsa-bind` API and LSP surface
- an ESM TypeScript wrapper under `dist/`
- no bundled `typescript-go` executable

## Runtime requirement

You must provide a compatible `typescript-go` (`tsgo`) executable yourself and
pass its path through `TsgoApiClient.spawn({ executable: "/path/to/tsgo" })`.

## Development

```bash
vp install
vp run -w build_wrapper
vp test run --config ./vite.config.ts npm/corsa_bind_node/ts/**/*.test.ts
```

Repository-level executable examples live under [`examples/`](../../examples/README.md),
including mock-client, virtual-document, distributed-orchestrator, and
real-`tsgo` snapshot samples.
