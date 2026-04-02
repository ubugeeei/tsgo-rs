import { spawnSync, type SpawnSyncOptions } from "node:child_process";
import { dirname, resolve } from "node:path";
import { fileURLToPath } from "node:url";

export const rootDir = resolve(dirname(fileURLToPath(import.meta.url)), "..");

interface RunCommandOptions {
  cwd?: string;
  env?: NodeJS.ProcessEnv;
  stdio?: SpawnSyncOptions["stdio"];
}

export interface CapturedCommandResult {
  signal: NodeJS.Signals | null;
  status: number | null;
  stderr: string;
  stdout: string;
}

export function runCommand(
  command: string,
  args: readonly string[],
  options: RunCommandOptions = {},
): void {
  const result = spawnSync(command, [...args], {
    cwd: options.cwd ?? rootDir,
    env: options.env ?? process.env,
    stdio: options.stdio ?? "inherit",
  });

  if (result.error) {
    throw result.error;
  }

  if (result.status !== 0) {
    throw new Error(
      `Command failed: ${command} ${args.join(" ")} (${result.status ?? result.signal ?? "unknown"})`,
    );
  }
}

export function runCommandCapture(
  command: string,
  args: readonly string[],
  options: Omit<RunCommandOptions, "stdio"> = {},
): CapturedCommandResult {
  const result = spawnSync(command, [...args], {
    cwd: options.cwd ?? rootDir,
    encoding: "utf8",
    env: options.env ?? process.env,
    stdio: "pipe",
  });

  if (result.error) {
    throw result.error;
  }

  return {
    signal: result.signal,
    status: result.status,
    stderr: result.stderr ?? "",
    stdout: result.stdout ?? "",
  };
}

export function assertCommandSucceeded(
  command: string,
  args: readonly string[],
  result: CapturedCommandResult,
): CapturedCommandResult {
  if (result.status === 0) {
    return result;
  }

  const output = [result.stdout.trim(), result.stderr.trim()].filter(Boolean).join("\n");
  throw new Error(
    `Command failed: ${command} ${args.join(" ")} (${result.status ?? result.signal ?? "unknown"})${output ? `\n${output}` : ""}`,
  );
}

export function sleep(ms: number): Promise<void> {
  return new Promise((resolveSleep) => setTimeout(resolveSleep, ms));
}

export function fail(error: unknown): never {
  if (error instanceof Error) {
    console.error(error.stack ?? error.message);
  } else {
    console.error(String(error));
  }
  process.exit(1);
}
