import { mkdirSync } from "node:fs";
import { resolve } from "node:path";

import { fail, rootDir, runCommand } from "./shared.ts";

function main(): void {
  const originDir = resolve(rootDir, "origin/typescript-go");
  const goCacheDir = resolve(rootDir, ".cache/go-build");
  const outputName = process.platform === "win32" ? "tsgo.exe" : "tsgo";
  const outputPath = resolve(rootDir, ".cache", outputName);

  mkdirSync(goCacheDir, { recursive: true });

  runCommand("go", ["build", "-o", outputPath, "./cmd/tsgo"], {
    cwd: originDir,
    env: {
      ...process.env,
      GOCACHE: goCacheDir,
    },
  });
}

try {
  main();
} catch (error) {
  fail(error);
}
