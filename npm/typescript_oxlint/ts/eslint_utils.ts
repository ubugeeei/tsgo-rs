import { getParserServices } from "./parser_services";
import { decorateRule } from "./plugin";
import type { ContextWithParserOptions } from "./types";

/**
 * Self-hosted `typescript-eslint`-style utility surface that swaps parser
 * service access over to tsgo-backed implementations.
 */
export const ESLintUtils = Object.freeze({
  RuleCreator(urlCreator: (ruleName: string) => string) {
    return (rule: any) => {
      const docs = rule.meta?.docs;
      return decorateRule({
        ...rule,
        meta: {
          ...rule.meta,
          docs: {
            ...docs,
            url: urlCreator(rule.name),
          },
        },
        defaultOptions: rule.defaultOptions ?? [],
      } as never);
    };
  },
  getParserServices(context: ContextWithParserOptions, allowWithoutFullTypeInformation = false) {
    return getParserServices(context, allowWithoutFullTypeInformation);
  },
});

export const RuleCreator = ESLintUtils.RuleCreator;
export { getParserServices } from "./parser_services";

export function applyDefault<
  Values extends readonly unknown[],
  Defaults extends readonly unknown[],
>(values: Values | undefined, defaults: Defaults): readonly unknown[] {
  return deepMerge(defaults, values ?? []) as readonly unknown[];
}

export function deepMerge<T>(base: T, override: unknown): T {
  if (Array.isArray(base) && Array.isArray(override)) {
    return base.map((value, index) => deepMerge(value, override[index])) as unknown as T;
  }
  if (isObject(base) && isObject(override)) {
    return Object.fromEntries(
      [...new Set([...Object.keys(base), ...Object.keys(override)])].map((key) => [
        key,
        deepMerge((base as any)[key], (override as any)[key]),
      ]),
    ) as T;
  }
  return (override ?? base) as T;
}

export function nullThrows<T>(
  value: T | null | undefined,
  message = "Expected value to be present",
): T {
  if (value == null) {
    throw new Error(message);
  }
  return value;
}

function isObject(value: unknown): value is Record<string, unknown> {
  return typeof value === "object" && value !== null && !Array.isArray(value);
}
