import { bench, describe } from "vitest";

import type { ApiMode } from "@tsgo-rs/tsgo-rs-node";

import { typescriptOxlintRules } from "typescript-oxlint/rules";

import {
  ensureBenchInputs,
  tsgoPath,
  typescriptOxlintConfigPath,
  typescriptOxlintFixtureDir,
  typescriptOxlintFilePath,
  typescriptOxlintSourceText,
  workspaceRoot,
} from "./support";

ensureBenchInputs();

const plusExpression = "lhs + rhs";
const sortExpression = "values.sort()";
const rejectExpression = 'Promise.reject(new Error("boom"))';

for (const mode of ["msgpack", "jsonrpc"] as const) {
  describe(`typescript-oxlint-rules ${mode}`, () => {
    const restrictContext = createContext(mode, [{ allowNumberAndString: false }]);
    const restrictVisitors = (typescriptOxlintRules["restrict-plus-operands"] as any).create(
      restrictContext,
    );
    const sortContext = createContext(mode);
    const sortVisitors = (typescriptOxlintRules["require-array-sort-compare"] as any).create(
      sortContext,
    );
    const rejectContext = createContext(mode);
    const rejectVisitors = (typescriptOxlintRules["prefer-promise-reject-errors"] as any).create(
      rejectContext,
    );

    bench("restrict-plus-operands visitor", () => {
      restrictVisitors.BinaryExpression(createPlusNode());
    });

    bench("require-array-sort-compare visitor", () => {
      sortVisitors.CallExpression(createSortNode());
    });

    bench("prefer-promise-reject-errors visitor", () => {
      rejectVisitors.CallExpression(createRejectNode());
    });
  });
}

function createContext(mode: ApiMode, options: readonly unknown[] = []) {
  return {
    cwd: typescriptOxlintFixtureDir,
    filename: typescriptOxlintFilePath,
    options,
    report() {},
    settings: {
      typescriptOxlint: {
        parserOptions: {
          project: [typescriptOxlintConfigPath],
          tsconfigRootDir: typescriptOxlintFixtureDir,
          tsgo: {
            executable: tsgoPath,
            cwd: workspaceRoot,
            cacheLifetimeMs: 60_000,
            mode,
          },
        },
      },
    },
    sourceCode: {
      text: typescriptOxlintSourceText,
      getAncestors() {
        return [];
      },
    },
  };
}

function createPlusNode() {
  const offset = typescriptOxlintSourceText.indexOf(plusExpression);
  return {
    type: "BinaryExpression",
    operator: "+",
    left: createIdentifier("lhs", offset, offset + 3),
    right: createIdentifier("rhs", offset + 6, offset + 9),
    range: [offset, offset + plusExpression.length] as const,
  };
}

function createSortNode() {
  const offset = typescriptOxlintSourceText.indexOf(sortExpression);
  const valuesStart = offset;
  const valuesEnd = valuesStart + "values".length;
  const sortStart = typescriptOxlintSourceText.indexOf("sort", valuesEnd);
  const sortEnd = sortStart + "sort".length;
  return {
    type: "CallExpression",
    callee: {
      type: "MemberExpression",
      computed: false,
      object: createIdentifier("values", valuesStart, valuesEnd),
      property: createIdentifier("sort", sortStart, sortEnd),
      range: [valuesStart, sortEnd] as const,
    },
    arguments: [],
    range: [offset, offset + sortExpression.length] as const,
  };
}

function createRejectNode() {
  const offset = typescriptOxlintSourceText.indexOf(rejectExpression);
  const promiseStart = offset;
  const promiseEnd = promiseStart + "Promise".length;
  const rejectStart = typescriptOxlintSourceText.indexOf("reject", promiseEnd);
  const rejectEnd = rejectStart + "reject".length;
  const errorStart = typescriptOxlintSourceText.indexOf("new Error", rejectEnd);
  const errorEnd = errorStart + 'new Error("boom")'.length;
  return {
    type: "CallExpression",
    callee: {
      type: "MemberExpression",
      computed: false,
      object: createIdentifier("Promise", promiseStart, promiseEnd),
      property: createIdentifier("reject", rejectStart, rejectEnd),
      range: [promiseStart, rejectEnd] as const,
    },
    arguments: [
      {
        type: "NewExpression",
        callee: createIdentifier("Error", errorStart + 4, errorStart + 9),
        arguments: [
          {
            type: "Literal",
            value: "boom",
            range: [errorStart + 10, errorEnd - 1] as const,
          },
        ],
        range: [errorStart, errorEnd] as const,
      },
    ],
    range: [offset, offset + rejectExpression.length] as const,
  };
}

function createIdentifier(name: string, pos: number, end: number) {
  return {
    type: "Identifier",
    name,
    range: [pos, end] as const,
  };
}
