# Release Guide

This document is the operational release guide for `corsa-bind`.

## Distribution Decisions

Public Rust crates:

- `corsa_bind_core`
- `corsa_bind_runtime`
- `corsa_bind_jsonrpc`
- `corsa_bind_client`
- `corsa_bind_lsp`
- `corsa_bind_orchestrator`
- `corsa_bind_rs`

Internal Rust crates:

- `corsa_bind_ref`
- `corsa_bind_node`

Public npm packages:

- `@corsa-bind/node` (`src/bindings/nodejs/corsa_bind_node`)
- `corsa-oxlint` (`src/bindings/nodejs/corsa_oxlint`)

The npm packages do not bundle the `typescript-go` executable. Consumers must
point them at a compatible `tsgo` binary at runtime.

`@corsa-bind/node` is built with `napi-rs`. The publish workflow now ships
the root package plus target-specific native binary packages for:

- `darwin-arm64`
- `darwin-x64`
- `linux-x64-gnu`
- `win32-x64-msvc`

Trusted publishing must be configured for each of those target-specific native
packages as well as the `@corsa-bind/node` root package.

The root package stays JS-only at publish time and resolves the correct native
binding through optional dependencies.

## Rust Publish Order

Publish crates in dependency order:

1. `corsa_bind_core`
2. `corsa_bind_runtime`
3. `corsa_bind_jsonrpc`
4. `corsa_bind_client`
5. `corsa_bind_lsp`
6. `corsa_bind_orchestrator`
7. `corsa_bind_rs`

## npm Publish Order

Publish npm packages in dependency order:

1. `@corsa-bind/node`
2. `corsa-oxlint`

## Dry Run

Local dry run:

```bash
vp run -w release_dry_run
```

This performs:

- `cargo package` for every public Rust crate
- a temporary workspace patch overlay so interdependent unpublished crates can be packaged before the first crates.io release
- staging of the JS-only `@corsa-bind/node` root package plus any locally available native binary packages
- `pnpm pack` for each publishable npm package so `workspace:*` ranges are rewritten exactly as they will be for publish
- `npm publish --dry-run <tarball>` for the packed npm tarballs

CI also runs the same release dry-run workflow.

## Release Checks

Before publishing Rust crates:

- `vp check`
- `cargo clippy --workspace --all-targets -- -D warnings`
- `vp run -w test`
- `vp run -w verify_origin`
- `vp run -w bench_verify`
- `cargo deny check advisories bans licenses sources`
- `vp run -w release_dry_run`

Before publishing npm packages, run the same gates plus a fresh `vp run -w build`.
The GitHub publish workflow fan-outs native binding builds per target, downloads
those `.node` artifacts into the publish job, and only then publishes the root
package and `corsa-oxlint`.

## Trusted Publishing

### crates.io

After the first manual release of each public crate, configure crates.io Trusted
Publishing to trust this repository and the [`publish-rust.yml`](../.github/workflows/publish-rust.yml)
workflow.

The workflow uses GitHub OIDC plus `rust-lang/crates-io-auth-action@v1`, so no
long-lived `CARGO_REGISTRY_TOKEN` secret is required after the initial release.

### npm

After the first manual publish of each npm package, configure npm Trusted
Publishing for each package with:

- GitHub organization or user: `ubugeeei`
- repository: `corsa-bind`
- workflow filename: `publish-npm.yml`
- environment: `release`

For `@corsa-bind/node`, also configure the same trusted publisher on each native
binary package:

- `@corsa-bind/node-darwin-arm64`
- `@corsa-bind/node-darwin-x64`
- `@corsa-bind/node-linux-x64-gnu`
- `@corsa-bind/node-win32-x64-msvc`

The npm workflow pins Node `24`, which satisfies npm's Trusted Publishing
minimum (`Node >= 22.14.0`, `npm >= 11.5.1`).

Once npm Trusted Publishing is working, update each package's npm settings to
`Require two-factor authentication and disallow tokens`.

## First Manual Publish

Both registries require an initial manual publish before OIDC-only trusted
publishing can take over.

### crates.io

```bash
cargo login
node --strip-types ./scripts/publish_rust.ts
```

This publishes the public crates in dependency order with the same sequencing
used by CI.

If crates.io rate-limits the first burst of new crates, the publish script now
waits until the reported retry time and continues automatically. If the process
stops midway, resume from the first missing crate:

```bash
CARGO_PUBLISH_START_AT=corsa_bind_rs node --strip-types ./scripts/publish_rust.ts
```

### npm

```bash
npm login
vp install
vp run -w build
NAPI_ARTIFACTS_DIR=./artifacts node --strip-types ./scripts/publish_npm.ts
```

This packs each workspace package through `pnpm pack`, then publishes the
resulting tarballs with `npm publish`, so the packed manifest already contains
real semver ranges instead of `workspace:*`.

For production releases, the npm publish script now refuses to publish the
`@corsa-bind/node` root package unless every configured native binding target is
present. Stage the `.node` artifacts from the build matrix into `./artifacts`
before running the first manual publish. `NAPI_REQUIRE_ALL_TARGETS=0` is still
available for local experimentation, but it is not production-safe for a real
release.

The trusted-publishing workflow follows the same order, but publishes the
target-specific native binding packages first and the JS-only
`@corsa-bind/node` root package after every required artifact is present.

If your npm account enforces 2FA, complete the interactive challenge during
this first manual publish.

If a publish partially succeeds, rerun from the first missing package:

```bash
NPM_PUBLISH_START_AT=corsa-oxlint NAPI_ARTIFACTS_DIR=./artifacts node --strip-types ./scripts/publish_npm.ts
```

## Changelog Expectations

Each public release should ship with GitHub release notes that call out:

- changed public crates
- any experimental-surface changes
- breaking changes or required upgrades
- benchmark or regression notes when performance-sensitive behavior changed

## Automation

Workflows:

- `CI`: quality, experimental-surface validation, real-`tsgo` smoke, and benchmark verification
- `Release Dry Run`: validates publishable artifacts without publishing them
- `Publish Rust`: crates.io trusted publish path for the public Rust crates after the initial manual release
- `Publish npm`: npm trusted publish path for the public npm packages after the initial manual release
- `Supply Chain`: runs dependency policy checks

The publish workflows are intentionally separate from the dry run so that
artifact validation stays cheap and safe on pull requests.

For dependency-policy and advisory handling, see [./supply_chain_policy.md](./supply_chain_policy.md).
