# tsgo-rs

Rust bindings, orchestration layers, and Node bindings for `typescript-go` over stdio.

> [!WARNING]
> This repository is still an early WIP.
> Public APIs, crate boundaries, benchmark budgets, and distributed orchestration details are still evolving.
> The current direction is production-oriented, but the project should not be treated as a frozen stable interface yet.

> [!IMPORTANT]
> `tsgo-rs` is intentionally built around upstream-supported `typescript-go`
> workflows. We follow `tsgo`'s recommended stdio/API/LSP integration points,
> keep `ref/typescript-go` as an exact upstream checkout, and preserve a strict
> `no forks, no patches` policy.

## What This Is

`tsgo-rs` is a multi-crate workspace for talking to `typescript-go` from Rust and Node.js without patching upstream.

In practice, that means:

- use `typescript-go` through the interfaces it already intends consumers to use
- track upstream by exact commit so behavior is reproducible and auditable
- never maintain a fork and never carry local patches against upstream `tsgo`
- implement hot paths in Rust, keep them zero-cost and high-performance, and
  expose them to JS through `napi-rs` so end users can author custom plugins
  and custom rules in JS/TS

Current focus:

- Full Rust-side stdio bindings for the tsgo API
- stdio LSP bindings with virtual-file support
- zero-cost-lean hot paths with msgpack-first defaults
- `napi-rs` bindings that surface Rust performance to JS/TS authoring workflows
- multi-process orchestration, cache reuse, and replicated state
- strict upstream pinning by exact `typescript-go` commit
- regression tests and benchmarks against the real pinned upstream server

## Current Status

- License: MIT
- Upstream policy: `ref/typescript-go` is pinned and tracked by exact commit, with no local patching
- Default API transport: `SyncMsgpackStdio`
- Runtime: custom in-house runtime, no `tokio`
- Fast-path bias: `CompactString`, `SmallVec`, `bumpalo`, `memchr`, `phf`, `FxHash`
- JS toolchain: `pnpm` + Vite+ (`vp`) with `oxfmt` / `oxlint`
- Node bindings: `npm/tsgo_rs_node` (`ESM-only` public JS surface)
- TS benchmark project: `bench`

Pinned upstream at the time of writing:

- Repository: `https://github.com/microsoft/typescript-go.git`
- Commit: `8a834dad086d6912b091e8b467e98499dab68cd9`
- Lock file: [`tsgo_ref.lock.toml`](./tsgo_ref.lock.toml)

## Workspace Layout

- `tsgo-rs-core`: shared errors, process handles, and fast-path primitives
- `tsgo-rs-jsonrpc`: stdio JSON-RPC framing and connection management
- `tsgo-rs-client`: typed tsgo stdio client bindings for JSON-RPC and msgpack
- `tsgo-rs-lsp`: LSP client support plus virtual-document overlays
- `tsgo-rs-orchestrator`: local orchestration, caching, replicated state, and Raft core
- `tsgo-rs-runtime`: lightweight custom runtime and task primitives
- `tsgo-rs-ref`: exact upstream pin, sync, and verification tooling
- `tsgo-rs`: top-level facade crate, mock server, and native benchmark binaries
- `npm/tsgo_rs_node`: `napi-rs` native bindings and the TypeScript wrapper package
- `npm/typescript_oxlint`: `typescript-eslint`-style compatibility layer for type-aware Oxlint JS plugins
- `bench`: Vitest benchmark project for the Node binding

For a detailed architecture walkthrough, design strategy, and implementation tips, see [docs/project_guide.md](./docs/project_guide.md).

## Quick Start

Sync and verify the pinned upstream checkout:

```bash
vp run -w sync_ref
vp run -w verify_ref
```

Install JS dependencies and build everything through Vite Task:

```bash
vp install
vp run -w build
vp check
```

## Type-Aware Oxlint

`typescript-oxlint` lets us write Oxlint JS plugins with a familiar
`typescript-eslint` authoring model while sourcing type information from the
pinned `tsgo` binary. The heavy lifting stays in Rust, then `napi-rs` binds
that implementation into JS so end users can keep writing custom plugins and
custom rules in JS/TS.

```ts
import { ESLintUtils } from "typescript-oxlint";

const createRule = ESLintUtils.RuleCreator((name) => `https://example.com/rules/${name}`);

export const noStringPlusNumber = createRule({
  name: "no-string-plus-number",
  meta: {
    type: "problem",
    docs: {
      description: "forbid string + number",
      requiresTypeChecking: true,
    },
    messages: {
      unexpected: "string plus number is forbidden",
    },
    schema: [],
  },
  defaultOptions: [],
  create(context) {
    const services = ESLintUtils.getParserServices(context);
    const checker = services.program.getTypeChecker();

    return {
      BinaryExpression(node) {
        if (node.operator !== "+") {
          return;
        }
        const left = checker.getTypeAtLocation(node.left);
        const right = checker.getTypeAtLocation(node.right);
        if (!left || !right) {
          return;
        }
        if (
          checker.typeToString(checker.getBaseTypeOfLiteralType(left) ?? left) === "string" &&
          checker.typeToString(checker.getBaseTypeOfLiteralType(right) ?? right) === "number"
        ) {
          context.report({ node, messageId: "unexpected" });
        }
      },
    };
  },
});
```

The rule-side type-aware config lives under `settings.typescriptOxlint`. Package
details and caveats are documented in [`npm/typescript_oxlint/README.md`](./npm/typescript_oxlint/README.md).

`typescript-oxlint/rules` exposes a TS-native type-aware rule set and plugin:

```ts
import { typescriptOxlintPlugin } from "typescript-oxlint/rules";

export default [
  {
    plugins: {
      typescript: typescriptOxlintPlugin,
    },
    rules: {
      "typescript/no-floating-promises": "error",
      "typescript/prefer-promise-reject-errors": "error",
      "typescript/restrict-plus-operands": ["error", { allowNumberAndString: false }],
    },
  },
];
```

The compatibility layer is self-hosted and does not depend on
`@typescript-eslint`. Upstream `tsgolint/internal/rules` is now used as a
parity target and drift oracle rather than as a runtime bridge.

## Example

```rust
use tsgo_rs::{
    api::{ApiClient, ApiSpawnConfig},
    runtime::block_on,
};

fn main() -> Result<(), tsgo_rs::TsgoError> {
    block_on(async {
        let client = ApiClient::spawn(
            ApiSpawnConfig::new(".cache/tsgo")
                .with_cwd("ref/typescript-go/_packages/api"),
        )
        .await?;

        let init = client.initialize().await?;
        println!("{}", init.current_directory);

        client.close().await?;
        Ok(())
    })
}
```

## Benchmarks

The repo ships two benchmark layers:

- Native Rust benchmark: `vp run -w bench_native`
- Node binding benchmark: `vp run -w bench_ts`
- `typescript-oxlint` checker benchmark: `vp test bench --config ./vite.config.ts bench/src/typescript_oxlint.bench.ts`
- `typescript-oxlint` native-rule benchmark: `vp test bench --config ./vite.config.ts bench/src/typescript_oxlint_rules.bench.ts`
- Combined benchmark + budget guard: `vp run -w bench`

The TS benchmark writes machine-readable output to `.cache/bench_ts.json`.
The native benchmark writes machine-readable output to `.cache/bench_native.json`.
The native Rust benchmark uses the real pinned tsgo binary through [`bench_real_tsgo`](./crates/tsgo_rs/src/bin/bench_real_tsgo/main.rs).

Latest native measurements are documented in [docs/performance.md](./docs/performance.md).
Benchmarking rationale, implementation notes, and usage tips are documented in [docs/benchmarking_guide.md](./docs/benchmarking_guide.md).
CI structure, local reproduction steps, and troubleshooting notes are documented in [docs/ci_guide.md](./docs/ci_guide.md).
On the pinned upstream commit and bundled datasets, `msgpack` was consistently faster than async JSON-RPC, which is why `ApiSpawnConfig::new()` defaults to `SyncMsgpackStdio`.

## Regression Strategy

The repository is intentionally aggressive about change detection because `typescript-go` is still unstable.

- `cargo test --workspace` includes mock-server integration tests, policy tests, and real-tsgo regression tests when `.cache/tsgo` is available
- `crates/tsgo_rs/tests/real_tsgo_baseline.rs` locks a real-server API summary to the pinned upstream commit
- `crates/tsgo_rs/tests/real_tsgo_regression.rs` checks both transports against the real pinned tsgo binary
- the real-tsgo regression suite includes a hot-path guard that fails if msgpack falls too far behind JSON-RPC on the same machine
- `vp run -w bench_native` and `vp run -w bench_ts` give repeatable transport-level measurements for Rust and Node
- `vp run -w bench_verify` regenerates both reports and fails if benchmark samples disappear or hot-path budgets regress
- `tsgo-rs-ref` enforces detached-HEAD exact-commit verification for `ref/typescript-go`
- CI structure and local reproduction notes live in [`docs/ci_guide.md`](./docs/ci_guide.md)

## Upstream Tracking

`typescript-go` is under heavy development, so reproducibility is treated as a hard requirement.

- exact commit metadata lives in [`tsgo_ref.lock.toml`](./tsgo_ref.lock.toml)
- sync and drift tooling lives in [`docs/tsgo_dependency.md`](./docs/tsgo_dependency.md)
- CI and local reproduction details live in [`docs/ci_guide.md`](./docs/ci_guide.md)
- `ref/typescript-go` must remain on detached `HEAD`
- dirty upstream worktrees fail verification

## Known Limitations

- This is still early WIP, so public APIs may change without notice.
- `printNode` is excluded from the default real-server benchmark suite because the pinned upstream `tsgo` commit can panic in `internal/printer` on real project data.
- The distributed layer currently includes an in-process Raft core; full network transport between nodes is not finished yet.
- Some binary API surfaces are still exposed as opaque encoded payloads rather than fully decoded Rust AST types.

## Development

Useful commands:

```bash
vp install
vp fmt
vp lint
vp test
vp check
vp run -w build
vp run -w bench
vp run -w bench_native
vp run -w bench_ts
vp run -w bench_verify
vp test run --config ./vite.config.ts
vp test bench --config ./vite.config.ts --outputJson .cache/bench_ts.json
vp run -w sync_ref
vp run -w verify_ref
```
