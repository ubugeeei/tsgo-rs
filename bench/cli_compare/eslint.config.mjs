import { fileURLToPath } from "node:url";
import { dirname, resolve } from "node:path";

import tsParser from "@typescript-eslint/parser";
import tsPlugin from "@typescript-eslint/eslint-plugin";

const tsconfig = process.env.TSGO_RS_BENCH_TSCONFIG;

if (!tsconfig) {
  throw new Error("TSGO_RS_BENCH_TSCONFIG is required");
}

const tsconfigPath = resolve(tsconfig);
const tsconfigRootDir = dirname(tsconfigPath);
const configDir = dirname(fileURLToPath(import.meta.url));
const basePath = resolve(configDir, "../..");

export default [
  {
    basePath,
    files: ["**/*.ts", "**/*.tsx", "**/*.mts", "**/*.cts"],
    languageOptions: {
      parser: tsParser,
      parserOptions: {
        project: [tsconfigPath],
        tsconfigRootDir,
        sourceType: "module",
        ecmaVersion: "latest"
      }
    },
    plugins: {
      "@typescript-eslint": tsPlugin
    },
    rules: {
      "@typescript-eslint/await-thenable": "error",
      "@typescript-eslint/no-floating-promises": "error",
      "@typescript-eslint/no-misused-promises": "error",
      "@typescript-eslint/no-unnecessary-condition": "error",
      "@typescript-eslint/no-unnecessary-type-assertion": "error"
    }
  }
];
