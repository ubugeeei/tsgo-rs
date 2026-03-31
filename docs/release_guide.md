# Release Guide

This document is the operational release guide for `tsgo-rs`.

## Distribution Decisions

Public Rust crates:

- `tsgo_rs_core`
- `tsgo_rs_runtime`
- `tsgo_rs_jsonrpc`
- `tsgo_rs_client`
- `tsgo_rs_lsp`
- `tsgo_rs_orchestrator`
- `tsgo_rs`

Internal Rust crates:

- `tsgo_rs_ref`
- `tsgo_rs_node`

Public npm packages:

- `@tsgo-rs/tsgo-rs-node` (`npm/tsgo_rs_node`)
- `typescript-oxlint` (`npm/typescript_oxlint`)

The npm packages do not bundle the `typescript-go` executable. Consumers must
point them at a compatible `tsgo` binary at runtime.

`@tsgo-rs/tsgo-rs-node` is built with `napi-rs`. The publish workflow currently
keeps the multi-platform fan-out step as an explicit placeholder, so runner-local
builds are wired up now and the cross-build packaging step can be dropped in
later without changing the release contract.

## Rust Publish Order

Publish crates in dependency order:

1. `tsgo_rs_core`
2. `tsgo_rs_runtime`
3. `tsgo_rs_jsonrpc`
4. `tsgo_rs_client`
5. `tsgo_rs_lsp`
6. `tsgo_rs_orchestrator`
7. `tsgo_rs`

## npm Publish Order

Publish npm packages in dependency order:

1. `@tsgo-rs/tsgo-rs-node`
2. `typescript-oxlint`

## Dry Run

Local dry run:

```bash
vp run -w release_dry_run
```

This performs:

- `cargo package` for every public Rust crate
- a temporary workspace patch overlay so interdependent unpublished crates can be packaged before the first crates.io release
- `pnpm pack` for each npm workspace package so `workspace:*` ranges are rewritten exactly as they will be for publish
- `npm publish --dry-run <tarball>` for the packed npm tarballs

CI also runs the same release dry-run workflow.

## Release Checks

Before publishing Rust crates:

- `vp check`
- `cargo clippy --workspace --all-targets -- -D warnings`
- `vp run -w test`
- `vp run -w verify_ref`
- `vp run -w bench_verify`
- `cargo deny check advisories bans licenses sources`
- `vp run -w release_dry_run`

Before publishing npm packages, run the same gates plus a fresh `vp run -w build`
on the runner or workstation that will create the native `napi-rs` artifact.

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
- repository: `tsgo-rs`
- workflow filename: `publish-npm.yml`
- environment: `release`

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
node ./scripts/publish_rust.mjs
```

This publishes the public crates in dependency order with the same sequencing
used by CI.

### npm

```bash
npm login
vp install
vp run -w build
node ./scripts/publish_npm.mjs
```

This packs each workspace package through `pnpm pack`, then publishes the
resulting tarballs with `npm publish`, so the packed manifest already contains
real semver ranges instead of `workspace:*`.

If your npm account enforces 2FA, complete the interactive challenge during
this first manual publish.

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
