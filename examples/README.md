# Examples

These examples are split into three groups:

- `examples/nodejs/*`: executable `@corsa-bind/node` samples
- `examples/rust/*`: executable Rust samples
- `examples/corsa_oxlint/*`: reusable `corsa-oxlint` rule/plugin/config samples

The `corsa_*` naming follows TypeScript's internal codenames as well:
`Corsa` refers to the native TypeScript 7 effort, while `Strada` refers to the
existing JS-based line. These examples are all oriented around the native
`typescript-go` side of that split.

## Prerequisites

Build the workspace packages and native bindings first:

```bash
vp install
vp run -w build
```

Run all smoke-tested examples:

```bash
vp run -w examples_smoke
```

Run only the Node / TypeScript example suite:

```bash
vp run -w examples_node_smoke
```

Run only the Rust example suite:

```bash
vp run -w examples_rust_smoke
```

Build the real pinned `tsgo` binary before running the real-snapshot examples:

```bash
vp run -w sync_origin
vp run -w verify_origin
vp run -w build_tsgo
vp run -w examples_real
```

Run the experimental distributed Rust example:

```bash
vp run -w examples_rust_experimental
```

## Minimal Start

These examples do not require a real `typescript-go` binary and are the best first touchpoints.

- `examples/nodejs/minimal_start.ts`: zero-binary start that combines virtual-document edits with the Rust-backed unsafe-type helpers
- `examples/nodejs/unsafe_type_flow.ts`: direct `isUnsafeAssignment()` / `isUnsafeReturn()` predicates for quick rule prototyping
- `examples/nodejs/virtual_document.ts`: focused in-memory document editing through `TsgoVirtualDocument`
- `examples/rust/minimal_start.rs`: smallest Rust facade example for `VirtualDocument`, `RequestId`, and `block_on()`
- `examples/rust/virtual_document.rs`: incremental and replace-style edits through the Rust `VirtualDocument`

Run one of them directly with:

```bash
pnpm --dir examples run minimal-start
cargo run -p corsa_bind_rs --example minimal_start
```

## Mock Binary Workflows

These examples use the repo-local `mock_tsgo` binary so you can exercise realistic API and LSP flows without building the real upstream server.

- `examples/nodejs/mock_client.ts`: high-level mock API roundtrip through `TsgoApiClient`
- `examples/nodejs/raw_calls.ts`: low-level `callJson()` / `callBinary()` escape hatches for custom endpoints
- `examples/nodejs/distributed_orchestrator.ts`: in-process distributed state replication for virtual documents
- `examples/rust/mock_client.rs`: typed snapshot, source-file, and type-string queries through the Rust API client
- `examples/rust/filesystem_callbacks.rs`: custom `ApiFileSystem` callbacks with a virtualized workspace
- `examples/rust/lsp_overlay.rs`: `LspClient` plus `LspOverlay` for `didOpen` / `didChange` / `didClose`
- `examples/rust/orchestrator_cache.rs`: local worker pooling, snapshot caching, and parallel fan-out through `ApiOrchestrator`
- `examples/rust/observer_events.rs`: structured `TsgoEvent` capture for cache eviction and operational telemetry

Run one of them directly with:

```bash
pnpm --dir examples run raw-calls
cargo run -p corsa_bind_rs --example lsp_overlay
```

## Real Pinned `tsgo`

These examples hit the exact upstream-pinned checkout under `origin/typescript-go`.

- `examples/nodejs/real_snapshot.ts`: opens the pinned project through `@corsa-bind/node` and fetches a real source file snapshot
- `examples/rust/real_snapshot.rs`: the Rust-side equivalent using the msgpack-first API client

Run one directly with:

```bash
pnpm --dir examples run real-snapshot
cargo run -p corsa_bind_rs --example real_snapshot
```

## Experimental Distributed

This example is intentionally separated because it requires the cargo feature.

- `examples/rust/distributed_orchestrator.rs`: replicated document and cached-result flow through `DistributedApiOrchestrator`

Run it with:

```bash
cargo run -p corsa_bind_rs --features experimental-distributed --example distributed_orchestrator
```

## `corsa-oxlint` Examples

- `examples/corsa_oxlint/custom_rule.ts`: custom type-aware rule using `ESLintUtils.getParserServices()`
- `examples/corsa_oxlint/custom_plugin.ts`: plugin wrapper around the custom rule
- `examples/corsa_oxlint/custom_rules_config.ts`: flat config using the custom plugin
- `examples/corsa_oxlint/native_rules_config.ts`: flat config using the built-in native rules
- `examples/corsa_oxlint/rule_tester.ts`: executable `RuleTester` example against the real pinned `tsgo` binary
- `examples/corsa_oxlint/native_rule_tester.ts`: executable `RuleTester` example for the built-in Rust-backed and TS-native rules
