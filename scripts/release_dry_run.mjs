import { mkdtempSync, rmSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { dirname, resolve } from "node:path";
import { spawnSync } from "node:child_process";
import { fileURLToPath } from "node:url";

import { npmPackages, publishPackedTarball } from "./npm_release_utils.mjs";

const rootDir = resolve(dirname(fileURLToPath(import.meta.url)), "..");
const crates = [
  {
    name: "tsgo_rs_core",
    path: resolve(rootDir, "crates/tsgo_rs_core"),
    patches: [],
  },
  {
    name: "tsgo_rs_runtime",
    path: resolve(rootDir, "crates/tsgo_rs_runtime"),
    patches: [],
  },
  {
    name: "tsgo_rs_jsonrpc",
    path: resolve(rootDir, "crates/tsgo_rs_jsonrpc"),
    patches: ["tsgo_rs_core", "tsgo_rs_runtime"],
  },
  {
    name: "tsgo_rs_client",
    path: resolve(rootDir, "crates/tsgo_rs_client"),
    patches: ["tsgo_rs_core", "tsgo_rs_jsonrpc", "tsgo_rs_runtime"],
  },
  {
    name: "tsgo_rs_lsp",
    path: resolve(rootDir, "crates/tsgo_rs_lsp"),
    patches: ["tsgo_rs_core", "tsgo_rs_jsonrpc", "tsgo_rs_runtime"],
  },
  {
    name: "tsgo_rs_orchestrator",
    path: resolve(rootDir, "crates/tsgo_rs_orchestrator"),
    patches: ["tsgo_rs_client", "tsgo_rs_core", "tsgo_rs_lsp", "tsgo_rs_runtime"],
  },
  {
    name: "tsgo_rs",
    path: resolve(rootDir, "crates/tsgo_rs"),
    patches: [
      "tsgo_rs_client",
      "tsgo_rs_core",
      "tsgo_rs_jsonrpc",
      "tsgo_rs_lsp",
      "tsgo_rs_orchestrator",
      "tsgo_rs_runtime",
    ],
  },
];
function run(command, args, cwd = rootDir) {
  const result = spawnSync(command, args, {
    cwd,
    stdio: "inherit",
    env: process.env,
  });
  if (result.status !== 0) {
    process.exit(result.status ?? 1);
  }
}

function normalizePath(path) {
  return path.replaceAll("\\", "/");
}

function patchConfigFor(crateName) {
  const crate = crates.find((candidate) => candidate.name === crateName);
  if (!crate) {
    throw new Error(`Unknown crate: ${crateName}`);
  }
  const patchLines = crate.patches.map((patchName) => {
    const dependency = crates.find((candidate) => candidate.name === patchName);
    if (!dependency) {
      throw new Error(`Unknown patch target: ${patchName}`);
    }
    return `${dependency.name} = { path = "${normalizePath(dependency.path)}" }`;
  });
  const configDir = mkdtempSync(resolve(tmpdir(), "tsgo-rs-release-dry-run-"));
  const configPath = resolve(configDir, "cargo-config.toml");
  const configBody = patchLines.length === 0 ? "" : `[patch.crates-io]\n${patchLines.join("\n")}\n`;
  writeFileSync(configPath, configBody, "utf8");
  return { configDir, configPath };
}

for (const crate of crates) {
  const { configDir, configPath } = patchConfigFor(crate.name);
  try {
    run("cargo", [
      "package",
      "--locked",
      "--allow-dirty",
      "--no-verify",
      "--config",
      configPath,
      "-p",
      crate.name,
    ]);
  } finally {
    rmSync(configDir, { recursive: true, force: true });
  }
}

for (const npmPackage of npmPackages) {
  publishPackedTarball(npmPackage, { dryRun: true });
}
