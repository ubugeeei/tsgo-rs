import { dirname, resolve } from "node:path";
import { spawnSync } from "node:child_process";
import { fileURLToPath } from "node:url";

const rootDir = resolve(dirname(fileURLToPath(import.meta.url)), "..");
const crates = [
  "tsgo_rs_core",
  "tsgo_rs_runtime",
  "tsgo_rs_jsonrpc",
  "tsgo_rs_client",
  "tsgo_rs_lsp",
  "tsgo_rs_orchestrator",
  "tsgo_rs",
];
const delayMs = Number(process.env.CARGO_PUBLISH_DELAY_MS ?? "30000");

function run(command, args) {
  const result = spawnSync(command, args, {
    cwd: rootDir,
    stdio: "inherit",
    env: process.env,
  });
  if (result.status !== 0) {
    process.exit(result.status ?? 1);
  }
}

for (const [index, crateName] of crates.entries()) {
  run("cargo", ["publish", "--locked", "-p", crateName]);
  if (index + 1 < crates.length && delayMs > 0) {
    await new Promise((resolveDelay) => setTimeout(resolveDelay, delayMs));
  }
}
