import { mkdtempSync, rmSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { resolve } from "node:path";

import {
  publishPackedTarball,
  corsaOxlintPackage,
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
    name: "corsa_bind_core",
    path: resolve(rootDir, "crates/corsa_bind_core"),
    patches: [],
  },
  {
    name: "corsa_bind_runtime",
    path: resolve(rootDir, "crates/corsa_bind_runtime"),
    patches: [],
  },
  {
    name: "corsa_bind_jsonrpc",
    path: resolve(rootDir, "crates/corsa_bind_jsonrpc"),
    patches: ["corsa_bind_core", "corsa_bind_runtime"],
  },
  {
    name: "corsa_bind_client",
    path: resolve(rootDir, "crates/corsa_bind_client"),
    patches: ["corsa_bind_core", "corsa_bind_jsonrpc", "corsa_bind_runtime"],
  },
  {
    name: "corsa_bind_lsp",
    path: resolve(rootDir, "crates/corsa_bind_lsp"),
    patches: ["corsa_bind_core", "corsa_bind_jsonrpc", "corsa_bind_runtime"],
  },
  {
    name: "corsa_bind_orchestrator",
    path: resolve(rootDir, "crates/corsa_bind_orchestrator"),
    patches: ["corsa_bind_client", "corsa_bind_core", "corsa_bind_lsp", "corsa_bind_runtime"],
  },
  {
    name: "corsa_bind_rs",
    path: resolve(rootDir, "crates/corsa_bind_rs"),
    patches: [
      "corsa_bind_client",
      "corsa_bind_core",
      "corsa_bind_jsonrpc",
      "corsa_bind_lsp",
      "corsa_bind_orchestrator",
      "corsa_bind_runtime",
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

  const configDir = mkdtempSync(resolve(tmpdir(), "corsa-bind-release-dry-run-"));
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
      for (const npmPackage of [...binaryPackages, rootPackage, corsaOxlintPackage]) {
        publishPackedTarball(npmPackage, { dryRun: true });
      }
    },
  );
}

await main().catch(fail);
