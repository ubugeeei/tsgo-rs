import { bench, describe } from "vitest";

import type { ApiMode } from "@tsgo-rs/node";

import type { TsgoNode, TsgoTypeCheckerShape } from "oxlint-plugin-typescript-go";
import { getParserServices } from "oxlint-plugin-typescript-go";

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

const expression = "lhs + rhs";
const expressionOffset = typescriptOxlintSourceText.indexOf(expression);
const leftNode = createNode(expressionOffset, expressionOffset + 3);
const rightNode = createNode(expressionOffset + 6, expressionOffset + 9);

const warmCheckers = {
  jsonrpc: getParserServices(createContext("jsonrpc") as never).program.getTypeChecker(),
  msgpack: getParserServices(createContext("msgpack") as never).program.getTypeChecker(),
} as const;

for (const mode of ["msgpack", "jsonrpc"] as const) {
  describe(`oxlint-plugin-typescript-go ${mode}`, () => {
    bench("parserServices init", () => {
      const services = getParserServices(createContext(mode) as never);
      const checker = services.program.getTypeChecker();
      typeText(checker, leftNode);
      typeText(checker, rightNode);
    });

    bench("typeAtLocation", () => {
      const checker = warmCheckers[mode];
      typeText(checker, leftNode);
      typeText(checker, rightNode);
    });
  });
}

function createContext(mode: ApiMode) {
  return {
    cwd: typescriptOxlintFixtureDir,
    filename: typescriptOxlintFilePath,
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
    },
  };
}

function createNode(pos: number, end: number): TsgoNode {
  return {
    fileName: typescriptOxlintFilePath,
    pos,
    end,
    range: [pos, end],
  };
}

function typeText(checker: TsgoTypeCheckerShape, node: TsgoNode): string | undefined {
  const type = checker.getTypeAtLocation(node);
  if (!type) {
    return undefined;
  }
  return checker.typeToString(checker.getBaseTypeOfLiteralType(type) ?? type);
}
