import { existsSync } from "node:fs";
import { resolve } from "node:path";

import { describe, it } from "vitest";

import { RuleTester } from "./rule_tester";
import { typescriptOxlintRules } from "./rules";

const workspaceRoot = resolve(import.meta.dirname, "../../..");
const realTsgoBinary = resolve(workspaceRoot, ".cache/tsgo");
const integrationCase = existsSync(realTsgoBinary) ? it : it.skip;

describe("typescript-oxlint native rule edges", () => {
  integrationCase("covers array, enum, promise, and sort edge cases", () => {
    const tester = createTester();

    tester.run("no-array-delete", typescriptOxlintRules["no-array-delete"] as never, {
      valid: [
        {
          code: "const record = { value: 1 }; delete record.value;",
        },
      ],
      invalid: [
        {
          code: "const values = [1, 2, 3]; const index = 1; delete values[index];",
          errors: [{ messageId: "unexpected" }],
        },
      ],
    });

    tester.run("no-mixed-enums", typescriptOxlintRules["no-mixed-enums"] as never, {
      valid: [
        {
          code: "enum Labels { A = 'two', B = 'three' }",
        },
      ],
      invalid: [
        {
          code: "const label = 'two' as const; enum Mixed { A = 1, B = label }",
          errors: [{ messageId: "mixed" }],
        },
      ],
    });

    tester.run("no-unsafe-unary-minus", typescriptOxlintRules["no-unsafe-unary-minus"] as never, {
      valid: [
        {
          code: "declare const value: bigint; const result = -value;",
        },
        {
          code: "declare const value: any; const result = -value;",
        },
      ],
      invalid: [
        {
          code: "declare const value: string | number; const result = -value;",
          errors: [{ messageId: "unaryMinus" }],
        },
      ],
    });

    tester.run(
      "prefer-promise-reject-errors",
      typescriptOxlintRules["prefer-promise-reject-errors"] as never,
      {
        valid: [
          {
            code: "new Promise((resolve, reject) => reject(new Error('boom')));",
          },
          {
            code: "Promise.reject();",
            options: [{ allowEmptyReject: true }],
          },
          {
            code: "Promise.reject(undefined as unknown);",
            options: [{ allowThrowingUnknown: true }],
          },
        ],
        invalid: [
          {
            code: "new Promise((resolve, reject) => reject('boom'));",
            errors: [{ messageId: "rejectAnError" }],
          },
        ],
      },
    );

    tester.run(
      "require-array-sort-compare",
      typescriptOxlintRules["require-array-sort-compare"] as never,
      {
        valid: [
          {
            code: "const values = ['b', 'a']; values.sort();",
          },
        ],
        invalid: [
          {
            code: "const values = ['b', 'a']; values.sort();",
            options: [{ ignoreStringArrays: false }],
            errors: [{ messageId: "requireCompare" }],
          },
        ],
      },
    );
  });
});

function createTester(): RuleTester {
  return new RuleTester({
    settings: {
      typescriptOxlint: {
        parserOptions: {
          tsgo: {
            executable: realTsgoBinary,
          },
        },
      },
    },
  } as never);
}
