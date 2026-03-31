import { spawnSync } from "node:child_process";
import { mkdtempSync, readdirSync, rmSync } from "node:fs";
import { tmpdir } from "node:os";
import { dirname, resolve } from "node:path";
import { fileURLToPath } from "node:url";

export const rootDir = resolve(dirname(fileURLToPath(import.meta.url)), "..");
const npmCommand = process.platform === "win32" ? "npm.cmd" : "npm";
const pnpmCommand = process.platform === "win32" ? "pnpm.cmd" : "pnpm";

export const npmPackages = [
  {
    name: "@tsgo-rs/tsgo-rs-node",
    path: resolve(rootDir, "npm/tsgo_rs_node"),
    access: "public",
  },
  {
    name: "typescript-oxlint",
    path: resolve(rootDir, "npm/typescript_oxlint"),
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

export function withPackedTarball(pkg, callback) {
  const packDir = mkdtempSync(resolve(tmpdir(), "tsgo-rs-npm-pack-"));
  try {
    run(pnpmCommand, ["pack", "--pack-destination", packDir], pkg.path);
    const tarballName = readdirSync(packDir).find((entry) => entry.endsWith(".tgz"));
    if (!tarballName) {
      throw new Error(`Failed to pack npm tarball for ${pkg.name}`);
    }
    return callback(resolve(packDir, tarballName));
  } finally {
    rmSync(packDir, { recursive: true, force: true });
  }
}

export function publishPackedTarball(pkg, { dryRun = false, tag } = {}) {
  return withPackedTarball(pkg, (tarballPath) => {
    const args = ["publish", tarballPath];
    if (pkg.access) {
      args.push("--access", pkg.access);
    }
    if (tag) {
      args.push("--tag", tag);
    }
    if (dryRun) {
      args.push("--dry-run");
    }
    run(npmCommand, args, rootDir);
  });
}

export function sleep(ms) {
  return new Promise((resolveSleep) => setTimeout(resolveSleep, ms));
}
