import { existsSync } from "node:fs";
import { resolve } from "node:path";

import { describe, expect, it } from "vitest";

import { defaultTsgoExecutable } from "./context";
import { ESLintUtils } from "./eslint_utils";
import { decorateRule, definePlugin } from "./plugin";
import { RuleTester } from "./rule_tester";

const workspaceRoot = resolve(import.meta.dirname, "../../..");
const realTsgoBinary = defaultTsgoExecutable(workspaceRoot);

describe("corsa-oxlint", () => {
  it("creates docs URLs through the typescript-eslint-compatible RuleCreator", () => {
    const createRule = ESLintUtils.RuleCreator((name) => `https://example.com/rules/${name}`);
    const rule = createRule({
      name: "no-demo",
      meta: {
        type: "problem",
        docs: {
          description: "demo rule",
        },
        messages: {
          demo: "demo",
        },
        schema: [],
      },
      defaultOptions: [],
      create() {
        return {};
      },
    });

    expect(((rule as any).meta as { docs: { url: string } }).docs.url).toBe(
      "https://example.com/rules/no-demo",
    );
  });

  it("wraps plugin rules with parserServices access", () => {
    const plugin = definePlugin({
      meta: { name: "corsa-oxlint-demo" },
      rules: {
        demo: {
          create(context) {
            expect(typeof (context as any).parserServices?.getTypeAtLocation).toBe("function");
            return {};
          },
        },
      },
    });

    expect(plugin.rules?.demo).toBeDefined();
  });

  it("hydrates parserOptions from settings.corsaOxlint", () => {
    let seen: Record<string, unknown> | undefined;
    const rule = decorateRule({
      meta: {
        messages: {
          demo: "demo",
        },
        schema: [],
      },
      create(context: any) {
        seen = {
          executable: context.parserOptions.tsgo?.executable,
          project: context.languageOptions?.parserOptions?.project,
          hasParserServices: "parserServices" in (context as object),
        };
        return {};
      },
    } as any);

    rule.create({
      cwd: workspaceRoot,
      filename: resolve(workspaceRoot, "fixture.ts"),
      languageOptions: {
        parserOptions: {},
      },
      report() {},
      settings: {
        corsaOxlint: {
          parserOptions: {
            project: ["tsconfig.json"],
            tsgo: {
              executable: realTsgoBinary,
            },
          },
        },
      },
      sourceCode: {
        text: "const fixture = 1;",
      },
    } as any);

    expect(seen).toEqual({
      executable: realTsgoBinary,
      project: ["tsconfig.json"],
      hasParserServices: true,
    });
  });

  it("propagates corsaOxlint settings through RuleTester", () => {
    let seen: Record<string, unknown> | undefined;
    const tester = new RuleTester();
    tester.run(
      "settings-roundtrip",
      {
        meta: {
          messages: {
            demo: "demo",
          },
          schema: [],
        },
        create(context: any) {
          seen = {
            languageExecutable: context.languageOptions?.parserOptions?.tsgo?.executable,
            parserExecutable: context.parserOptions?.tsgo?.executable,
            settingsExecutable: context.settings?.corsaOxlint?.parserOptions?.tsgo?.executable,
          };
          return {};
        },
      } as any,
      {
        valid: [
          {
            code: "const value = 1;",
            settings: {
              corsaOxlint: {
                parserOptions: {
                  tsgo: {
                    executable: realTsgoBinary,
                  },
                },
              },
            },
          },
        ],
        invalid: [],
      },
    );

    expect(seen).toEqual({
      languageExecutable: realTsgoBinary,
      parserExecutable: realTsgoBinary,
      settingsExecutable: realTsgoBinary,
    });
  });

  const integrationCase = existsSync(realTsgoBinary) ? it : it.skip;

  integrationCase("runs a type-aware custom rule through oxlint RuleTester", () => {
    const createRule = ESLintUtils.RuleCreator((name) => `https://example.com/rules/${name}`);
    const rule = createRule({
      name: "no-string-plus-number",
      meta: {
        type: "problem",
        docs: {
          description: "reject string plus number",
          recommended: "recommended",
          requiresTypeChecking: true,
        },
        messages: {
          unexpected: "string plus number is forbidden",
        },
        schema: [],
      },
      defaultOptions: [],
      create(context: any) {
        const services = ESLintUtils.getParserServices(context);
        const checker = services.program.getTypeChecker();
        return {
          BinaryExpression(node: any) {
            if (node.operator !== "+") {
              return;
            }
            const left = normalize(checker.getTypeAtLocation(node.left));
            const right = normalize(checker.getTypeAtLocation(node.right));
            if (!left || !right) {
              return;
            }
            if (left === "string" && right === "number") {
              context.report({ node, messageId: "unexpected" });
            }
          },
        };

        function normalize(type: any): string | undefined {
          if (!type) {
            return undefined;
          }
          const normalized = checker.getBaseTypeOfLiteralType(type) ?? type;
          return checker.typeToString(normalized);
        }
      },
    });

    const tester = new RuleTester();
    tester.run("no-string-plus-number", rule as any, {
      valid: [
        {
          code: "const result = 1 + 2;",
          settings: {
            corsaOxlint: {
              parserOptions: {
                tsgo: {
                  executable: realTsgoBinary,
                },
              },
            },
          },
        },
      ],
      invalid: [
        {
          code: 'const lhs = "value"; const rhs = 1; const result = lhs + rhs;',
          errors: [{ messageId: "unexpected" }],
          settings: {
            corsaOxlint: {
              parserOptions: {
                tsgo: {
                  executable: realTsgoBinary,
                },
              },
            },
          },
        },
      ],
    });
  });
});
