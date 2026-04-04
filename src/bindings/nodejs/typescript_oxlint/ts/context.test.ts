import { mkdtempSync, mkdirSync, rmSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { dirname, join, resolve } from "node:path";

import { afterEach, describe, expect, it } from "vitest";

import {
  defaultTsgoExecutable,
  resolveProjectConfig,
  resolveTypeAwareParserOptions,
} from "./context";

const cleanupDirs = new Set<string>();
const normalizePathSeparators = (path: string) => path.replaceAll("\\", "/");

afterEach(() => {
  for (const dir of cleanupDirs) {
    rmSync(dir, { force: true, recursive: true });
  }
  cleanupDirs.clear();
});

describe("context", () => {
  it("merges settings.typescriptOxlint parser options ahead of Oxlint defaults", () => {
    const resolved = resolveTypeAwareParserOptions({
      cwd: "/repo",
      filename: "/repo/src/demo.ts",
      languageOptions: {
        parserOptions: {
          tsgo: {
            mode: "jsonrpc",
          },
        },
      },
      settings: {
        typescriptOxlint: {
          parserOptions: {
            project: ["tsconfig.json"],
            tsgo: {
              executable: "/repo/.cache/tsgo",
            },
          },
        },
      },
      sourceCode: {
        text: "const demo = 1;",
      },
    } as any);

    expect(resolved.project).toEqual(["tsconfig.json"]);
    expect(resolved.tsgo).toEqual({
      executable: "/repo/.cache/tsgo",
      mode: "jsonrpc",
    });
  });

  it("creates a default project when projectService is enabled from settings", () => {
    const workspace = mkdtempSync(join(tmpdir(), "oxlint-plugin-typescript-go-context-"));
    cleanupDirs.add(workspace);
    const filename = resolve(workspace, "src/demo.ts");
    mkdirSync(dirname(filename), { recursive: true });
    writeFileSync(filename, "export const demo = 1;\n");

    const resolved = resolveProjectConfig({
      cwd: workspace,
      filename,
      settings: {
        typescriptOxlint: {
          parserOptions: {
            projectService: {
              allowDefaultProject: ["*.ts"],
            },
            tsgo: {
              executable: resolve(workspace, ".cache/tsgo"),
            },
          },
        },
      },
      sourceCode: {
        text: "export const demo = 1;\n",
      },
    } as any);

    expect(normalizePathSeparators(resolved.configPath)).toContain(
      ".cache/typescript_oxlint/default/",
    );
    expect(resolved.runtime.executable).toBe(resolve(workspace, ".cache/tsgo"));
  });

  it("resolves the platform-specific default tsgo executable", () => {
    expect(defaultTsgoExecutable("/repo", "linux")).toBe(resolve("/repo", ".cache/tsgo"));
    expect(defaultTsgoExecutable("/repo", "win32")).toBe(resolve("/repo", ".cache/tsgo.exe"));
  });
});
