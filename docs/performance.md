# Performance

`tsgo-rs` ships with two benchmark layers:

- Native real-tsgo benchmark: `vp run -w bench_native`
- Node binding benchmark: `vp run -w bench_ts`
- Combined benchmark + budget guard: `vp run -w bench_verify`

For day-to-day formatting, linting, and testing, prefer `vp fmt`, `vp lint`,
and `vp check`. Those go through Vite+'s `oxfmt` / `oxlint` toolchain. The
`vp run -w ...` commands below are the specialized benchmark entrypoints.

The TS benchmark writes its report to `.cache/bench_ts.json`.
The native benchmark writes its report to `.cache/bench_native.json`.

The native runner is still the main source of truth for transport-level speed because it measures the Rust client directly against the pinned upstream worker.

## Native Runner

The native benchmark runner is the `bench_real_tsgo` binary:

```bash
cargo run --release -p tsgo-rs --bin bench_real_tsgo -- \
  --cold-iterations 5 \
  --warm-iterations 20 \
  --json-output .cache/bench_native.json
```

Warm scenarios perform one untimed warm-up call before sampling.

## Datasets

| dataset      | files |   bytes |  lines | config                                          |
| ------------ | ----: | ------: | -----: | ----------------------------------------------- |
| `ast`        |    29 | 630,429 | 14,653 | `ref/typescript-go/_packages/ast/tsconfig.json` |
| `api`        |    31 | 278,806 |  7,097 | `ref/typescript-go/_packages/api/tsconfig.json` |
| `_extension` |    13 |  78,255 |  2,022 | `ref/typescript-go/_extension/tsconfig.json`    |

## 2026-03-30 Native Bench

All numbers below are median milliseconds from:

```bash
vp run -w bench_native
```

| mode      | dataset      | spawn+initialize | parseConfigFile | updateSnapshot cold | updateSnapshot warm | getDefaultProject | getSourceFile | getStringType | typeToString |
| --------- | ------------ | ---------------: | --------------: | ------------------: | ------------------: | ----------------: | ------------: | ------------: | -----------: |
| `jsonrpc` | `ast`        |           20.385 |           0.205 |              28.934 |               0.075 |             0.040 |         0.389 |         0.025 |        0.025 |
| `jsonrpc` | `api`        |           19.846 |           0.289 |              32.262 |               0.070 |             0.040 |         0.113 |         0.024 |        0.024 |
| `jsonrpc` | `_extension` |           19.678 |           0.194 |              44.434 |               0.068 |             0.030 |         0.083 |         0.029 |        0.022 |
| `msgpack` | `ast`        |            6.546 |           0.166 |              16.104 |               0.052 |             0.020 |         0.167 |         0.011 |        0.010 |
| `msgpack` | `api`        |            6.226 |           0.266 |              22.039 |               0.050 |             0.024 |         0.046 |         0.013 |        0.013 |
| `msgpack` | `_extension` |            6.671 |           0.172 |              33.712 |               0.046 |             0.012 |         0.033 |         0.015 |        0.011 |

## TS Benchmark Notes

`vp run -w bench_ts` runs the Node binding through Vitest benchmark mode and emits `.cache/bench_ts.json`.
`vp run -w bench_verify` regenerates both benchmark reports and then runs a guard test over `.cache/bench_ts.json` and `.cache/bench_native.json`.

The current Vitest bench summary is useful for relative ranking but the JSON files are the better artifacts to inspect when tracking regressions over time.

## Notes

- `ApiSpawnConfig::new()` defaults to `SyncMsgpackStdio`, because it is still consistently ahead on the measured real-tsgo paths.
- `getSourceFile` benefits strongly from msgpack because async JSON-RPC has to carry binary payloads through JSON framing.
- `bench/src/report_guard.test.ts` fails when benchmark samples go missing or when the measured hot paths drift past the configured budget.
- `crates/tsgo_rs/tests/real_tsgo_baseline.rs` pins the real upstream API summary for the locked `tsgo` commit.
- `printNode` is intentionally excluded from the default native suite at the pinned upstream commit because the real `tsgo` server can still panic inside `internal/printer` on real project data.
