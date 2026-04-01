import { fileURLToPath } from "node:url";

import { describe, expect, it } from "vitest";

import {
  findNativeBenchEntry,
  findTsBenchEntry,
  readNativeBenchEntries,
  readTsBenchEntries,
} from "./report";

describe("bench report helpers", () => {
  it("flattens Vitest benchmark groups", () => {
    const entries = readTsBenchEntries(
      fileURLToPath(new URL("../fixtures/ts_bench_report.json", import.meta.url)),
    );

    expect(entries).toEqual([
      {
        mode: "msgpack",
        name: "spawn+initialize",
        median: 5.12,
        mean: 5.21,
        sampleCount: 90,
        rme: 1.4,
      },
      {
        mode: "jsonrpc",
        name: "spawn+initialize",
        median: 21.4,
        mean: 21.8,
        sampleCount: 24,
        rme: 1.7,
      },
    ]);
    expect(findTsBenchEntry(entries, "msgpack", "spawn+initialize").median).toBe(5.12);
  });

  it("loads native benchmark rows", () => {
    const entries = readNativeBenchEntries(
      fileURLToPath(new URL("../fixtures/native_bench_report.json", import.meta.url)),
    );

    expect(entries).toHaveLength(2);
    expect(findNativeBenchEntry(entries, "msgpack", "api", "spawn_initialize").medianMs).toBe(5.4);
  });
});
