# tsgo Dependency Management

`typescript-go` is managed as a pinned git dependency via [tsgo_ref.lock.toml](../tsgo_ref.lock.toml).

Core policy:

- `tsgo-rs` follows upstream-supported `typescript-go` integration points.
- `tsgo-rs` does not maintain a fork of `typescript-go`.
- `tsgo-rs` does not patch upstream `typescript-go`.
- Upstream changes are adopted by updating the pinned commit and adapting our bindings around that exact revision.

Rules:

- The authoritative upstream is `ref/typescript-go`.
- The lock file records repository, exact commit hash, tree hash, committer timestamp, author, and subject.
- `ref/typescript-go` must remain on a detached `HEAD` at the exact locked commit.
- A dirty worktree fails verification.
- `sync` refuses to touch an existing checkout when the configured remote does not match the locked upstream.

Workflow:

1. `cargo run -p tsgo-rs-ref -- sync`
2. `cargo run -p tsgo-rs-ref -- verify`
3. When intentionally updating upstream, move `ref/typescript-go` to the new commit and run `cargo run -p tsgo-rs-ref -- pin-current`

This keeps reproduction commit-exact and leaves an auditable metadata trail for every upstream bump.
