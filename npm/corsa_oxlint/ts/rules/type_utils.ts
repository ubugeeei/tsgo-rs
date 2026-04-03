import type { Node } from "@oxlint/plugins";

import { ESLintUtils } from "../eslint_utils";
import { isIdentifierNamed, memberPropertyName, stripChainExpression } from "./ast";
import type { ContextWithParserOptions, TsgoType, TsgoTypeCheckerShape } from "../types";

export function checkerFor(context: ContextWithParserOptions): TsgoTypeCheckerShape {
  return ESLintUtils.getParserServices(context).program.getTypeChecker();
}

export function typeAtNode(
  context: ContextWithParserOptions,
  node: Node | { readonly range: readonly [number, number] },
): TsgoType | undefined {
  return checkerFor(context).getTypeAtLocation(node as Node);
}

export function baseTypeAtNode(
  context: ContextWithParserOptions,
  node: Node | { readonly range: readonly [number, number] },
): TsgoType | undefined {
  const type = typeAtNode(context, node);
  if (!type) {
    return undefined;
  }
  return checkerFor(context).getBaseTypeOfLiteralType(type) ?? type;
}

export function symbolTypeAtNode(
  context: ContextWithParserOptions,
  node: Node | { readonly range: readonly [number, number] },
): TsgoType | undefined {
  const checker = checkerFor(context);
  const symbol = checker.getSymbolAtLocation(node as Node);
  if (!symbol) {
    return undefined;
  }
  return checker.getTypeOfSymbol(symbol) ?? checker.getDeclaredTypeOfSymbol(symbol);
}

export function typeTextAtNode(
  context: ContextWithParserOptions,
  node: Node | { readonly range: readonly [number, number] },
): string | undefined {
  const type = baseTypeAtNode(context, node);
  return type ? checkerFor(context).typeToString(type) : undefined;
}

export function symbolTypeTextAtNode(
  context: ContextWithParserOptions,
  node: Node | { readonly range: readonly [number, number] },
): string | undefined {
  const type = symbolTypeAtNode(context, node);
  if (!type) {
    return undefined;
  }
  const checker = checkerFor(context);
  return checker.typeToString(checker.getBaseTypeOfLiteralType(type) ?? type);
}

export function propertyNamesOfNode(
  context: ContextWithParserOptions,
  node: Node | { readonly range: readonly [number, number] },
): readonly string[] {
  const checker = checkerFor(context);
  const names = new Set<string>();
  for (const type of [baseTypeAtNode(context, node), symbolTypeAtNode(context, node)]) {
    if (!type) {
      continue;
    }
    for (const property of checker.getPropertiesOfType(type)) {
      names.add(property.name);
    }
  }
  return [...names];
}

export function isPromiseLikeNode(
  context: ContextWithParserOptions,
  node: Node | { readonly range: readonly [number, number] },
): boolean {
  const current = stripChainExpression(node as any) as any;
  if (current?.type === "NewExpression" && isIdentifierNamed(current.callee, "Promise")) {
    return true;
  }
  if (
    current?.type === "CallExpression" &&
    memberPropertyName(current.callee) === "resolve" &&
    isIdentifierNamed((current.callee as any).object, "Promise")
  ) {
    return true;
  }
  for (const text of [typeTextAtNode(context, node), symbolTypeTextAtNode(context, node)]) {
    if (text?.includes("Promise") || text?.includes("Thenable")) {
      return true;
    }
  }
  const properties = new Set(propertyNamesOfNode(context, node));
  return properties.has("then");
}

export function isArrayLikeNode(
  context: ContextWithParserOptions,
  node: Node | { readonly range: readonly [number, number] },
): boolean {
  const current = stripChainExpression(node as any) as any;
  if (current?.type === "ArrayExpression") {
    return true;
  }
  for (const text of typeTextsAtNode(context, node)) {
    if (
      text &&
      (text.endsWith("[]") ||
        text.startsWith("Array<") ||
        text.startsWith("ReadonlyArray<") ||
        (text.startsWith("[") && text.endsWith("]")))
    ) {
      return true;
    }
  }
  return false;
}

export function isStringLikeNode(
  context: ContextWithParserOptions,
  node: Node | { readonly range: readonly [number, number] },
): boolean {
  return typeTextsAtNode(context, node).some((text) => classifyTypeText(text) === "string");
}

export function isErrorLikeNode(
  context: ContextWithParserOptions,
  node: Node | { readonly range: readonly [number, number] },
): boolean {
  const current = stripChainExpression(node as any) as any;
  if (current?.type === "NewExpression") {
    const callee = stripChainExpression(current.callee);
    const identifier = callee?.type === "Identifier" ? callee.name : memberPropertyName(callee);
    if (identifier?.endsWith("Error")) {
      return true;
    }
  }
  for (const text of typeTextsAtNode(context, node)) {
    if (text === "Error" || text?.endsWith("Error")) {
      return true;
    }
  }
  const properties = new Set(propertyNamesOfNode(context, node));
  return properties.has("message") && properties.has("name");
}

export function isNumberLikeNode(
  context: ContextWithParserOptions,
  node: Node | { readonly range: readonly [number, number] },
): boolean {
  return typeTextsAtNode(context, node).some((text) => classifyTypeText(text) === "number");
}

export function isBigIntLikeNode(
  context: ContextWithParserOptions,
  node: Node | { readonly range: readonly [number, number] },
): boolean {
  return typeTextsAtNode(context, node).some((text) => classifyTypeText(text) === "bigint");
}

export function isAnyLikeNode(
  context: ContextWithParserOptions,
  node: Node | { readonly range: readonly [number, number] },
): boolean {
  const current = stripChainExpression(node as any) as any;
  if (current?.type === "TSAsExpression" && current.typeAnnotation?.type === "TSAnyKeyword") {
    return true;
  }
  return typeTextsAtNode(context, node).some((text) => classifyTypeText(text) === "any");
}

export function isUnknownLikeNode(
  context: ContextWithParserOptions,
  node: Node | { readonly range: readonly [number, number] },
): boolean {
  const current = stripChainExpression(node as any) as any;
  if (current?.type === "TSAsExpression" && current.typeAnnotation?.type === "TSUnknownKeyword") {
    return true;
  }
  return typeTextsAtNode(context, node).some((text) => classifyTypeText(text) === "unknown");
}

export function typeTextsAtNode(
  context: ContextWithParserOptions,
  node: Node | { readonly range: readonly [number, number] },
): readonly string[] {
  const values = new Set<string>();
  const checker = checkerFor(context);
  collectTexts(baseTypeAtNode(context, node));
  collectTexts(symbolTypeAtNode(context, node));
  return [...values];

  function collectTexts(type: TsgoType | undefined): void {
    if (!type) {
      return;
    }
    const texts = Array.isArray(type.texts) ? type.texts : [];
    for (const text of [...texts, checker.typeToString(type)]) {
      if (text) {
        values.add(text);
      }
    }
  }
}

export function classifyTypeText(
  text: string | undefined,
): "any" | "bigint" | "boolean" | "nullish" | "number" | "regexp" | "string" | "unknown" | "other" {
  if (!text) {
    return "other";
  }
  if (text === "any") {
    return "any";
  }
  if (text === "unknown" || text === "never") {
    return "unknown";
  }
  if (text === "string" || isQuotedStringLiteral(text)) {
    return "string";
  }
  if (text === "number" || /^-?\d+(\.\d+)?$/.test(text)) {
    return "number";
  }
  if (text === "bigint" || /^-?\d+n$/.test(text)) {
    return "bigint";
  }
  if (text === "boolean" || text === "true" || text === "false") {
    return "boolean";
  }
  if (text === "null" || text === "undefined" || text.includes("null |")) {
    return "nullish";
  }
  if (text.includes("RegExp")) {
    return "regexp";
  }
  return "other";
}

function isQuotedStringLiteral(text: string): boolean {
  return (
    (text.startsWith('"') && text.endsWith('"')) ||
    (text.startsWith("'") && text.endsWith("'")) ||
    (text.startsWith("`") && text.endsWith("`"))
  );
}
