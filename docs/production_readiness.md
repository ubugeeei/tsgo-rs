# Production Readiness Guide

This document is the short operational checklist for running `tsgo-rs` in production-style environments.

## Scope

The current production target is:

- local Rust and Node API clients
- LSP stdio integrations
- local worker orchestration and cache reuse

The following remains experimental:

- the `experimental-distributed` cargo feature
- the in-process Raft replication layer
- upstream endpoints called out as unstable by this repository

## Default Safety Controls

The default runtime configuration now includes:

- per-request timeout: `30s`
- graceful shutdown timeout: `2s`
- bounded outbound queue capacity: `256`
- unstable upstream endpoints disabled by default

These defaults can be overridden through:

- `ApiSpawnConfig`
- `LspSpawnConfig`
- `ApiOrchestratorConfig`

## Recommended Settings

For long-lived services:

- keep `request_timeout` enabled
- reduce `outbound_capacity` if you prefer earlier backpressure
- tune `max_cached_snapshots` and `max_cached_results` to fit process memory budgets
- wire a `TsgoObserver` into spawn/orchestrator configs so timeouts and evictions reach your telemetry stack
- leave unstable upstream endpoints disabled unless you have a concrete need and a rollback plan

For editor-like integrations:

- use stable cache keys for snapshots
- prewarm a small worker fleet instead of spawning per request
- treat the distributed orchestrator as experimental unless you are actively developing it

## Release Checklist

- `vp check`
- `cargo clippy --workspace --all-targets -- -D warnings`
- `vp run -w test`
- `cargo test -p tsgo_rs --no-default-features --test orchestrator`
- `cargo test -p tsgo_rs --features experimental-distributed --test orchestrator`
- `vp run -w bench_verify`
- `vp run -w verify_ref`
- `cargo deny check advisories bans licenses sources`
- `vp run -w release_dry_run`

## Cross-Platform Expectations

The main quality workflow is intended to stay green on:

- Linux
- macOS
- Windows

Real `tsgo` smoke coverage now runs across the supported OS matrix, while the
heavier benchmark verification remains concentrated in the Ubuntu benchmark job.
