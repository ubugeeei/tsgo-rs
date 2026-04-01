# Examples

These examples are split into two groups:

- `examples/node/*`: executable `@tsgo-rs/node` samples
- `examples/typescript_oxlint/*`: reusable `typescript-oxlint` rule/plugin/config samples

## Prerequisites

Build the workspace packages and native bindings first:

```bash
vp install
vp run -w build
```

Run the smoke-tested examples:

```bash
vp run -w examples_smoke
```

Build the real pinned `tsgo` binary before running the real snapshot example:

```bash
vp run -w sync_ref
vp run -w verify_ref
vp run -w build_tsgo
vp run -w examples_real
```

## Node Binding Examples

- `node/mock_client.ts`: talks to the repo-local mock tsgo binary
- `node/virtual_document.ts`: applies incremental virtual document edits
- `node/distributed_orchestrator.ts`: opens and replicates virtual documents across the in-process cluster
- `node/real_snapshot.ts`: opens the real pinned `typescript-go` project and fetches a source file snapshot

## typescript-oxlint Examples

- `typescript_oxlint/custom_rule.ts`: custom type-aware rule using `ESLintUtils.getParserServices()`
- `typescript_oxlint/custom_plugin.ts`: plugin wrapper around the custom rule
- `typescript_oxlint/custom_rules_config.ts`: flat config using the custom plugin
- `typescript_oxlint/native_rules_config.ts`: flat config using the built-in native rules
- `typescript_oxlint/rule_tester.ts`: executable `RuleTester` example against the real pinned `tsgo` binary
