import { mkdtempSync, rmSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { resolve } from "node:path";

import {
  publishPackedTarball,
  typescriptOxlintPackage,
  withStagedNodeBindingPackages,
} from "./npm_release_utils.ts";
import { fail, rootDir, runCommand } from "./shared.ts";

interface CrateSpec {
  name: string;
  path: string;
  patches: string[];
}

const crates: CrateSpec[] = [
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

function normalizePath(path: string): string {
  return path.replaceAll("\\", "/");
}

function patchConfigFor(crateName: string): { configDir: string; configPath: string } {
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

async function main(): Promise<void> {
  for (const crate of crates) {
    const { configDir, configPath } = patchConfigFor(crate.name);
    try {
      runCommand(
        "cargo",
        ["package", "--locked", "--allow-dirty", "--config", configPath, "-p", crate.name],
        { cwd: rootDir },
      );
    } finally {
      rmSync(configDir, { recursive: true, force: true });
    }
  }

  await withStagedNodeBindingPackages(
    { requireAllTargets: false },
    async ({ binaryPackages, rootPackage }) => {
      for (const npmPackage of [...binaryPackages, rootPackage, typescriptOxlintPackage]) {
        publishPackedTarball(npmPackage, { dryRun: true });
      }
    },
  );
}

await main().catch(fail);
