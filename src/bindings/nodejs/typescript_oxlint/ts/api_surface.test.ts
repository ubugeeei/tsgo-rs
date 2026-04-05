import { describe, expect, it } from "vitest";

import * as main from "./index";
import * as compatEntry from "./oxlint_compat";
import * as rules from "./rules";
import * as tsestreeEntry from "./ts_estree";

describe("api surface", () => {
  it("re-exports the compatibility entrypoint", () => {
    expect(typeof compatEntry.oxlintCompat.config).toBe("function");
    expect(compatEntry.oxlintCompat.parser.meta.name).toBe("oxlint-plugin-corsa/parser");
  });

  it("re-exports ts-estree helpers from the root entry", () => {
    expect(main.TSESTree.AST_NODE_TYPES.Program).toBe("Program");
    expect(tsestreeEntry.AST_NODE_TYPES.Identifier).toBe("Identifier");
  });

  it("re-exports the native rules surface from both entrypoints", () => {
    expect(typeof main.rules.typescriptOxlintPlugin).toBe("object");
    expect(rules.implementedNativeRuleNames).toContain("restrict-plus-operands");
  });

  it("re-exports Rust-backed utility helpers from the root entry", () => {
    expect(main.Utils.classifyTypeText("Promise<string>")).toBe("other");
    expect(main.Utils.isPromiseLikeTypeTexts(["Promise<string>"])).toBe(true);
    expect(main.Utils.splitTypeText("string | number & bigint")).toEqual([
      "string",
      "number",
      "bigint",
    ]);
  });
});
