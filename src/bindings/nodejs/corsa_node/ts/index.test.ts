import { existsSync } from "node:fs";
import { resolve } from "node:path";

import { describe, expect, it } from "vitest";

import {
  TsgoApiClient,
  TsgoDistributedOrchestrator,
  TsgoVirtualDocument,
  classifyTypeText,
  isAnyLikeTypeTexts,
  isArrayLikeTypeTexts,
  isErrorLikeTypeTexts,
  isPromiseLikeTypeTexts,
  splitTopLevelTypeText,
  splitTypeText,
  isUnsafeAssignment,
  isUnsafeReturn,
} from "./index";

const workspaceRoot = resolve(import.meta.dirname, "../../..");
const executableSuffix = process.platform === "win32" ? ".exe" : "";
const mockBinary = resolve(workspaceRoot, `target/debug/mock_tsgo${executableSuffix}`);
const realBinary = resolve(workspaceRoot, `.cache/tsgo${executableSuffix}`);
const realDataset = resolve(workspaceRoot, "ref/typescript-go/_packages/api/tsconfig.json");
const realTsgoReady = existsSync(realBinary) && existsSync(realDataset);

describe("TsgoApiClient", () => {
  it("evaluates Rust-backed unsafe type flow predicates", () => {
    expect(
      isUnsafeAssignment({
        sourceTypeTexts: ["Set<any>"],
        targetTypeTexts: ["Set<string>"],
      }),
    ).toBe(true);
    expect(
      isUnsafeAssignment({
        sourceTypeTexts: ["any"],
        targetTypeTexts: ["unknown"],
      }),
    ).toBe(false);
    expect(
      isUnsafeReturn({
        sourceTypeTexts: ["Promise<any>"],
        targetTypeTexts: ["Promise<string>"],
      }),
    ).toBe(true);
  });

  it("exposes Rust-backed type-text utilities", () => {
    expect(classifyTypeText('"value"')).toBe("string");
    expect(classifyTypeText("42n")).toBe("bigint");
    expect(splitTopLevelTypeText("Promise<string | number> | null", "|")).toEqual([
      "Promise<string | number>",
      "null",
    ]);
    expect(splitTypeText("string | Promise<Array<number>> & undefined")).toEqual([
      "string",
      "Promise<Array<number>>",
      "undefined",
    ]);
    expect(isArrayLikeTypeTexts(["ReadonlyArray<string>"])).toBe(true);
    expect(isPromiseLikeTypeTexts(["Promise<string>"])).toBe(true);
    expect(isPromiseLikeTypeTexts([], ["then"])).toBe(true);
    expect(isErrorLikeTypeTexts(["TypeError"])).toBe(true);
    expect(isAnyLikeTypeTexts(["any"])).toBe(true);
  });

  it("roundtrips through the mock tsgo binary", () => {
    const client = TsgoApiClient.spawn({
      executable: mockBinary,
      cwd: workspaceRoot,
      mode: "jsonrpc",
    });

    try {
      const init = client.initialize();
      expect(init.currentDirectory).toBe(workspaceRoot);

      const snapshot = client.updateSnapshot({
        openProject: "/workspace/tsconfig.json",
      });
      const project = snapshot.projects[0];
      expect(project).toBeDefined();

      const sourceFile = client.getSourceFile(
        snapshot.snapshot,
        project.id,
        "/workspace/src/index.ts",
      );
      expect(Buffer.from(sourceFile ?? []).toString("utf8")).toBe("source-file");
      expect(client.callJson<string>("ping")).toBe("pong");
      const sourceViaGeneric = client.callBinary("getSourceFile", {
        snapshot: snapshot.snapshot,
        project: project.id,
        file: "/workspace/src/index.ts",
      });
      expect(Buffer.from(sourceViaGeneric ?? []).toString("utf8")).toBe("source-file");

      const stringType = client.getStringType(snapshot.snapshot, project.id);
      expect(stringType.id).toBe("t0000000000000010");
      expect(client.typeToString(snapshot.snapshot, project.id, stringType.id)).toBe("type:string");

      client.releaseHandle(snapshot.snapshot);
    } finally {
      client.close();
    }
  });

  for (const mode of ["msgpack", "jsonrpc"] as const) {
    const realCase = realTsgoReady ? it : it.skip;

    realCase(`keeps real ${mode} snapshots alive across follow-up calls`, () => {
      const client = TsgoApiClient.spawn({
        executable: realBinary,
        cwd: workspaceRoot,
        mode,
      });

      try {
        client.initialize();
        const config = client.parseConfigFile(realDataset);
        const snapshot = client.updateSnapshot({ openProject: realDataset });
        const project = snapshot.projects[0];
        const primaryFile =
          config.fileNames.find((fileName) => !fileName.endsWith(".d.ts")) ?? config.fileNames[0];

        expect(project).toBeDefined();
        expect(primaryFile).toBeDefined();
        expect(client.getSourceFile(snapshot.snapshot, project.id, primaryFile)).not.toBeNull();
        const stringType = client.getStringType(snapshot.snapshot, project.id);
        expect(client.typeToString(snapshot.snapshot, project.id, stringType.id)).toBe("string");

        client.releaseHandle(snapshot.snapshot);
      } finally {
        client.close();
      }
    });
  }
});

describe("TsgoVirtualDocument", () => {
  it("tracks incremental virtual file changes", () => {
    const document = TsgoVirtualDocument.untitled(
      "/virtual/demo.ts",
      "typescript",
      "const value = 1;\n",
    );
    document.applyChanges([
      {
        range: {
          start: { line: 0, character: 14 },
          end: { line: 0, character: 15 },
        },
        text: "2",
      },
    ]);

    expect(document.version).toBe(2);
    expect(document.text).toBe("const value = 2;\n");
    expect(document.state().uri).toContain("untitled:");
  });
});

describe("TsgoDistributedOrchestrator", () => {
  it("replicates virtual documents after leader election", () => {
    const cluster = new TsgoDistributedOrchestrator(["n1", "n2", "n3"]);
    expect(cluster.campaign("n1")).toBe(1);

    const document = TsgoVirtualDocument.inMemory(
      "cluster",
      "/main.ts",
      "typescript",
      "let value = 1;",
    );
    cluster.openVirtualDocument(document.state());
    const updated = cluster.changeVirtualDocument(document.uri, [
      {
        range: {
          start: { line: 0, character: 12 },
          end: { line: 0, character: 13 },
        },
        text: "2",
      },
    ]);

    expect(updated.text).toBe("let value = 2;");
    expect(cluster.document("n2", document.uri)?.text).toBe("let value = 2;");
  });
});
