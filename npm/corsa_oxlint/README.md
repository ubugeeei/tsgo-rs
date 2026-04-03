# corsa-oxlint

`corsa-oxlint` is a self-hosted `typescript-eslint`-style framework for
building Oxlint JS plugins with real type information powered by `tsgo`.

The name is a deliberate nod to TypeScript's own codenames: `Corsa` is the
native TypeScript 7 effort, while `Strada` is the existing JS-based line.
`corsa-oxlint` is meant to read as the Oxlint-facing package for the native
`typescript-go` side of that split.

> [!WARNING]
> This package is still an early WIP.
> The core direction is stable, but the API surface will keep moving while
> `typescript-go`, Oxlint's JS plugin APIs, and the surrounding benchmarks are
> still evolving.

## What It Does

- exposes `ESLintUtils.RuleCreator()` and `getParserServices()` backed by `tsgo`
- preserves the familiar subpath layout without depending on `@typescript-eslint`
- binds Rust-implemented hot paths into JS through `napi-rs`
- lets custom Oxlint rules query types and symbols from JS or TS
- ships a `RuleTester` wrapper that injects temp projects and type-aware config
- ships a growing TS-native ruleset under `corsa-oxlint/rules`

The design goal is simple: performance-critical pieces live in Rust, `napi-rs`
bridges them into Node, and end users still get to author custom plugins and
custom rules in plain JS/TS.

## Configuration

Oxlint does not expose arbitrary parser options at runtime, so
`corsa-oxlint` reads its type-aware settings from
`settings.corsaOxlint`.

```ts
import { ESLintUtils } from "corsa-oxlint";

const createRule = ESLintUtils.RuleCreator((name) => `https://example.com/rules/${name}`);

export const noStringPlusNumber = createRule({
  name: "no-string-plus-number",
  meta: {
    type: "problem",
    docs: {
      description: "forbid string + number",
      requiresTypeChecking: true,
    },
    messages: {
      unexpected: "string plus number is forbidden",
    },
    schema: [],
  },
  defaultOptions: [],
  create(context) {
    const services = ESLintUtils.getParserServices(context);
    const checker = services.program.getTypeChecker();

    return {
      BinaryExpression(node) {
        if (node.operator !== "+") {
          return;
        }
        const left = checker.getTypeAtLocation(node.left);
        const right = checker.getTypeAtLocation(node.right);
        if (!left || !right) {
          return;
        }
        const leftText = checker.typeToString(checker.getBaseTypeOfLiteralType(left) ?? left);
        const rightText = checker.typeToString(checker.getBaseTypeOfLiteralType(right) ?? right);
        if (leftText === "string" && rightText === "number") {
          context.report({ node, messageId: "unexpected" });
        }
      },
    };
  },
});
```

```js
export default [
  {
    settings: {
      corsaOxlint: {
        parserOptions: {
          project: ["./tsconfig.json"],
          tsconfigRootDir: import.meta.dirname,
          tsgo: {
            executable: "./.cache/tsgo",
            mode: "msgpack",
            requestTimeoutMs: 30000,
          },
        },
      },
    },
  },
];
```

## Native Rules

`corsa-oxlint/rules` exports the TS-native rule set and plugin surface.
Rule parity is tracked against upstream `tsgolint/internal/rules`, but the
runtime implementation lives entirely in this package.

```ts
import { corsaOxlintPlugin } from "corsa-oxlint/rules";

export default [
  {
    plugins: {
      typescript: corsaOxlintPlugin,
    },
    rules: {
      "typescript/no-floating-promises": "error",
      "typescript/prefer-promise-reject-errors": "error",
      "typescript/restrict-plus-operands": ["error", { allowNumberAndString: false }],
    },
  },
];
```

Current native coverage includes:

- `await-thenable`
- `no-array-delete`
- `no-base-to-string`
- `no-floating-promises`
- `no-for-in-array`
- `no-implied-eval`
- `no-mixed-enums`
- `no-unsafe-assignment`
- `no-unsafe-return`
- `no-unsafe-unary-minus`
- `only-throw-error`
- `prefer-find`
- `prefer-includes`
- `prefer-promise-reject-errors`
- `prefer-regexp-exec`
- `prefer-string-starts-ends-with`
- `require-array-sort-compare`
- `restrict-plus-operands`
- `use-unknown-in-catch-callback-variable`

The remaining upstream rules stay listed in `pendingNativeRuleNames`, and
`native_rules.test.ts` fails if implemented + pending drift away from the
tracked upstream rule list.

## Runtime Safety Controls

The underlying `@corsa-bind/node` client now exposes a few production-oriented
runtime controls:

- `requestTimeoutMs`
- `shutdownTimeoutMs`
- `outboundCapacity`
- `allowUnstableUpstreamCalls`

Leaving `allowUnstableUpstreamCalls` unset keeps unstable upstream endpoints
such as `printNode` disabled by default.

## Development

```bash
vp install
vp run -w build_corsa_oxlint
vp fmt
vp lint
vp check
vp test run --config ./vite.config.ts npm/corsa_oxlint/ts/**/*.test.ts
vp test bench --config ./vite.config.ts bench/src/corsa_oxlint.bench.ts
vp test bench --config ./vite.config.ts bench/src/corsa_oxlint_rules.bench.ts
```

Repository-level examples live under [`examples/`](../../examples/README.md),
including custom-rule, custom-plugin, and native-rules flat-config samples.
