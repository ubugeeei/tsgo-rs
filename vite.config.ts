import { dirname, resolve } from "node:path";
import { fileURLToPath } from "node:url";

import { defineConfig } from "vite-plus";

const rootDir = dirname(fileURLToPath(import.meta.url));
const nodePackageDir = resolve(rootDir, "npm/tsgo_rs_node");
const typescriptOxlintDir = resolve(rootDir, "npm/typescript_oxlint");
const typescriptOxlintSourceDir = resolve(typescriptOxlintDir, "ts");
const generatedNodeArtifacts = [
  "npm/tsgo_rs_node/index.d.ts",
  "npm/tsgo_rs_node/index.js",
  "npm/tsgo_rs_node/ts/**/*.d.ts",
  "npm/tsgo_rs_node/ts/**/*.js",
  "npm/tsgo_rs_node/ts/**/*.js.map",
];
const lintIgnorePatterns = [
  ...generatedNodeArtifacts,
  "bench/fixtures/**",
  "npm/tsgo_rs_node/ts/**/*.test.ts",
];
const noopCommand = 'node -e "process.exit(0)"';

export default defineConfig({
  fmt: {
    ignorePatterns: generatedNodeArtifacts,
  },
  pack: {
    clean: true,
    deps: {
      neverBundle: ["@tsgo-rs/node"],
      skipNodeModulesBundle: true,
    },
    dts: true,
    entry: ["npm/typescript_oxlint/ts/**/*.ts", "!npm/typescript_oxlint/ts/**/*.test.ts"],
    fixedExtension: false,
    format: "esm",
    outDir: resolve(typescriptOxlintDir, "dist"),
    root: typescriptOxlintSourceDir,
    sourcemap: true,
    tsconfig: resolve(typescriptOxlintDir, "tsconfig.json"),
    unbundle: true,
  },
  resolve: {
    alias: {
      "@tsgo-rs/node": resolve(nodePackageDir, "ts/index.ts"),
      "oxlint-plugin-typescript-go/ast-utils": resolve(typescriptOxlintDir, "ts/ast_utils.ts"),
      "oxlint-plugin-typescript-go/eslint-utils": resolve(
        typescriptOxlintDir,
        "ts/eslint_utils.ts",
      ),
      "oxlint-plugin-typescript-go/json-schema": resolve(typescriptOxlintDir, "ts/json_schema.ts"),
      "oxlint-plugin-typescript-go/rule-tester": resolve(typescriptOxlintDir, "ts/rule_tester.ts"),
      "oxlint-plugin-typescript-go/rules": resolve(typescriptOxlintDir, "ts/rules/index.ts"),
      "oxlint-plugin-typescript-go/ts-eslint": resolve(typescriptOxlintDir, "ts/ts_eslint.ts"),
      "oxlint-plugin-typescript-go/ts-estree": resolve(typescriptOxlintDir, "ts/ts_estree.ts"),
      "oxlint-plugin-typescript-go": resolve(typescriptOxlintDir, "ts/index.ts"),
    },
  },
  lint: {
    ignorePatterns: lintIgnorePatterns,
    options: {
      typeAware: true,
      typeCheck: true,
    },
  },
  run: {
    tasks: {
      sync_ref: {
        command: "cargo run -p tsgo_rs_ref -- sync",
      },
      verify_ref: {
        command: "cargo run -p tsgo_rs_ref -- verify",
      },
      build: {
        command: noopCommand,
        dependsOn: ["build_mock", "build_wrapper", "build_typescript_oxlint"],
      },
      build_ci: {
        command: noopCommand,
        dependsOn: ["build_mock", "build_wrapper_ci", "build_typescript_oxlint_ci"],
      },
      build_rust: {
        command: "cargo build --workspace",
      },
      build_mock: {
        command: "cargo build -p tsgo_rs --bin mock_tsgo",
      },
      build_tsgo: {
        command: "node --strip-types ./scripts/build_tsgo.ts",
      },
      build_node_debug: {
        command: "napi build --platform",
        cwd: "npm/tsgo_rs_node",
        dependsOn: ["build_rust"],
      },
      build_node_release: {
        command: "napi build --platform --release",
        cwd: "npm/tsgo_rs_node",
        dependsOn: ["build_rust"],
      },
      build_typescript_oxlint: {
        command: "vp pack",
        dependsOn: ["build_wrapper"],
      },
      build_typescript_oxlint_ci: {
        command: "vp pack",
        dependsOn: ["build_wrapper_ci"],
      },
      build_wrapper: {
        command:
          "vp pack index.ts types.ts --dts --format esm --out-dir ../dist --sourcemap --tsconfig ../tsconfig.json --root . --deps.neverBundle ../index.js",
        cwd: "npm/tsgo_rs_node/ts",
        dependsOn: ["build_node_release"],
      },
      build_wrapper_ci: {
        command:
          "vp pack index.ts types.ts --dts --format esm --out-dir ../dist --sourcemap --tsconfig ../tsconfig.json --root . --deps.neverBundle ../index.js",
        cwd: "npm/tsgo_rs_node/ts",
        dependsOn: ["build_node_debug"],
      },
      lint_rust: {
        command: "cargo clippy --workspace --all-targets -- -D warnings",
      },
      fmt_rust: {
        cache: false,
        command: "cargo fmt --all",
      },
      fmt_check_rust: {
        command: "cargo fmt --all --check",
      },
      test: {
        command: noopCommand,
        dependsOn: ["test_rust", "test_rust_experimental", "test_ts", "examples_smoke"],
      },
      test_rust: {
        command: "cargo test --workspace",
      },
      test_rust_experimental: {
        command: "cargo test -p tsgo_rs --no-default-features --test orchestrator",
        dependsOn: ["test_rust_experimental_feature"],
      },
      test_rust_experimental_feature: {
        command: "cargo test -p tsgo_rs --features experimental-distributed --test orchestrator",
      },
      test_ts: {
        command: "vp test run --config ./vite.config.ts",
        dependsOn: ["build_mock", "build_node_debug"],
      },
      bench: {
        command: noopCommand,
        dependsOn: ["bench_verify"],
      },
      bench_native: {
        command:
          "cargo run --release -p tsgo_rs --bin bench_real_tsgo -- --cold-iterations 5 --warm-iterations 20 --json-output .cache/bench_native.json",
        dependsOn: ["build_tsgo"],
      },
      bench_native_deep: {
        command:
          "cargo run --release -p tsgo_rs --bin bench_real_tsgo -- --cold-iterations 10 --warm-iterations 80 --json-output .cache/bench_native_deep.json",
        dependsOn: ["build_tsgo"],
      },
      bench_tooling_setup: {
        command: noopCommand,
        dependsOn: ["bench_tooling_setup_ref", "bench_tooling_setup_cli_compare"],
      },
      bench_tooling_setup_ref: {
        command: "npm install --no-fund --no-audit",
        cwd: "ref/typescript-go",
      },
      bench_tooling_setup_cli_compare: {
        command: "npm install --no-fund --no-audit",
        cwd: "bench/cli_compare",
      },
      bench_tooling_compare: {
        command:
          "cargo run --release -p tsgo_rs --bin bench_tooling_compare -- --iterations 10 --warmup-iterations 2 --json-output .cache/bench_tooling_compare.json",
        dependsOn: ["build_tsgo", "bench_tooling_setup"],
      },
      bench_ts: {
        command: "vp test bench --config ./vite.config.ts --outputJson .cache/bench_ts.json",
        dependsOn: ["build_tsgo", "build_node_release"],
      },
      bench_verify: {
        command:
          "TSGO_REQUIRE_BENCH_REPORTS=1 vp test run --config ./vite.config.ts bench/src/report_guard.test.ts",
        dependsOn: ["bench_native", "bench_ts"],
      },
      release_dry_run: {
        command: "node --strip-types ./scripts/release_dry_run.ts",
        dependsOn: ["build"],
      },
      examples_node_smoke: {
        command: "pnpm run smoke",
        cwd: "examples",
        dependsOn: ["build"],
      },
      examples_node_real: {
        command: "pnpm run real",
        cwd: "examples",
        dependsOn: ["build", "sync_ref", "verify_ref", "build_tsgo"],
      },
      examples_rust_smoke: {
        command: "node --strip-types ./scripts/run_rust_examples.ts smoke",
        dependsOn: ["build_mock"],
      },
      examples_rust_real: {
        command: "node --strip-types ./scripts/run_rust_examples.ts real",
        dependsOn: ["sync_ref", "verify_ref", "build_tsgo"],
      },
      examples_rust_experimental: {
        command: "node --strip-types ./scripts/run_rust_examples.ts experimental",
        dependsOn: ["build_mock"],
      },
      examples_smoke: {
        command: noopCommand,
        dependsOn: ["examples_node_smoke", "examples_rust_smoke"],
      },
      examples_real: {
        command: noopCommand,
        dependsOn: ["examples_node_real", "examples_rust_real"],
      },
    },
  },
  test: {
    environment: "node",
    include: ["bench/src/**/*.test.ts", "npm/**/ts/**/*.test.ts"],
    benchmark: {
      include: ["bench/src/**/*.bench.ts"],
      exclude: ["ref/**"],
      includeSamples: true,
    },
  },
});
