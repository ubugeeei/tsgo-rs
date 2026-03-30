import { existsSync, readFileSync } from "node:fs";
import { resolve } from "node:path";

export type TsBenchMode = "jsonrpc" | "msgpack";

export type TsBenchEntry = {
  readonly mode: TsBenchMode;
  readonly name: string;
  readonly median: number;
  readonly mean: number;
  readonly sampleCount: number;
  readonly rme: number;
};

export type NativeBenchEntry = {
  readonly mode: TsBenchMode;
  readonly dataset: string;
  readonly scenario: string;
  readonly medianMs: number;
  readonly meanMs: number;
  readonly p95Ms: number;
};

type TsBenchGroup = {
  readonly fullName: string;
  readonly benchmarks: readonly {
    readonly name: string;
    readonly median?: number;
    readonly mean?: number;
    readonly sampleCount?: number;
    readonly rme?: number;
  }[];
};

type TsBenchFile = {
  readonly groups?: readonly TsBenchGroup[];
};

type NativeBenchReport = {
  readonly rows?: readonly NativeBenchEntry[];
};

const workspaceRoot = resolve(import.meta.dirname, "../..");
export const nativeBenchReportPath = resolve(workspaceRoot, ".cache/bench_native.json");
export const tsBenchReportPath = resolve(workspaceRoot, ".cache/bench_ts.json");

export function hasBenchReports(): boolean {
  return existsSync(nativeBenchReportPath) && existsSync(tsBenchReportPath);
}

export function readTsBenchEntries(
  reportPath: string = tsBenchReportPath,
): readonly TsBenchEntry[] {
  const report = JSON.parse(readFileSync(reportPath, "utf8")) as {
    readonly files?: readonly TsBenchFile[];
  };
  return (report.files ?? []).flatMap((file) =>
    (file.groups ?? []).flatMap((group) => {
      const mode = parseMode(group.fullName);
      return group.benchmarks.map((benchmark) => ({
        mode,
        name: benchmark.name,
        median: benchmark.median ?? 0,
        mean: benchmark.mean ?? 0,
        sampleCount: benchmark.sampleCount ?? 0,
        rme: benchmark.rme ?? 0,
      }));
    }),
  );
}

export function readNativeBenchEntries(
  reportPath: string = nativeBenchReportPath,
): readonly NativeBenchEntry[] {
  const report = JSON.parse(readFileSync(reportPath, "utf8")) as NativeBenchReport;
  return report.rows ?? [];
}

export function findTsBenchEntry(
  entries: readonly TsBenchEntry[],
  mode: TsBenchMode,
  name: string,
): TsBenchEntry {
  const entry = entries.find((candidate) => {
    return candidate.mode === mode && candidate.name === name;
  });
  if (!entry) {
    throw new Error(`missing ts benchmark ${mode}:${name}`);
  }
  return entry;
}

export function findNativeBenchEntry(
  entries: readonly NativeBenchEntry[],
  mode: TsBenchMode,
  dataset: string,
  scenario: string,
): NativeBenchEntry {
  const entry = entries.find((candidate) => {
    return (
      candidate.mode === mode && candidate.dataset === dataset && candidate.scenario === scenario
    );
  });
  if (!entry) {
    throw new Error(`missing native benchmark ${mode}:${dataset}:${scenario}`);
  }
  return entry;
}

function parseMode(fullName: string): TsBenchMode {
  if (fullName.endsWith(" msgpack")) {
    return "msgpack";
  }
  if (fullName.endsWith(" jsonrpc")) {
    return "jsonrpc";
  }
  throw new Error(`unknown benchmark group: ${fullName}`);
}
