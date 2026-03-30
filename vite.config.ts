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
      neverBundle: ["@tsgo-rs/tsgo-rs-node"],
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
      "@tsgo-rs/tsgo-rs-node": resolve(nodePackageDir, "ts/index.ts"),
      "typescript-oxlint/ast-utils": resolve(typescriptOxlintDir, "ts/ast_utils.ts"),
      "typescript-oxlint/eslint-utils": resolve(typescriptOxlintDir, "ts/eslint_utils.ts"),
      "typescript-oxlint/json-schema": resolve(typescriptOxlintDir, "ts/json_schema.ts"),
      "typescript-oxlint/rule-tester": resolve(typescriptOxlintDir, "ts/rule_tester.ts"),
      "typescript-oxlint/rules": resolve(typescriptOxlintDir, "ts/rules/index.ts"),
      "typescript-oxlint/ts-eslint": resolve(typescriptOxlintDir, "ts/ts_eslint.ts"),
      "typescript-oxlint/ts-estree": resolve(typescriptOxlintDir, "ts/ts_estree.ts"),
      "typescript-oxlint": resolve(typescriptOxlintDir, "ts/index.ts"),
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
        command: "cargo run -p tsgo-rs-ref -- sync",
      },
      verify_ref: {
        command: "cargo run -p tsgo-rs-ref -- verify",
      },
      build: {
        command: noopCommand,
        dependsOn: ["build_mock", "build_tsgo", "build_wrapper", "build_typescript_oxlint"],
      },
      build_rust: {
        command: "cargo build --workspace",
      },
      build_mock: {
        command: "cargo build -p tsgo-rs --bin mock_tsgo",
      },
      build_tsgo: {
        command: "go build -o ../../.cache/tsgo ./cmd/tsgo",
        cwd: "ref/typescript-go",
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
      build_wrapper: {
        command:
          "vp pack index.ts types.ts --dts --format esm --out-dir ../dist --sourcemap --tsconfig ../tsconfig.json --root . --deps.neverBundle ../index.js",
        cwd: "npm/tsgo_rs_node/ts",
        dependsOn: ["build_node_release"],
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
        dependsOn: ["test_rust", "test_ts"],
      },
      test_rust: {
        command: "cargo test --workspace",
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
          "cargo run --release -p tsgo-rs --bin bench_real_tsgo -- --cold-iterations 5 --warm-iterations 20 --json-output .cache/bench_native.json",
        dependsOn: ["build_tsgo"],
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
