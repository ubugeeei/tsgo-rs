import {
  calleePropertyName,
  isIdentifierNamed,
  isLiteralString,
  stripChainExpression,
} from "./ast";
import { createNativeRule } from "./rule_creator";
import { classifyTypeText, isStringLikeNode, typeTextsAtNode } from "./type_utils";

const knownSafeObjectTypes = new Set([
  "Date",
  "Error",
  "EvalError",
  "RangeError",
  "ReferenceError",
  "RegExp",
  "SyntaxError",
  "TypeError",
  "URIError",
  "URL",
  "URLSearchParams",
]);

export const noBaseToStringRule = createNativeRule(
  "no-base-to-string",
  {
    docs: {
      description: "Disallow stringifying values that fall back to Object.prototype.toString().",
    },
    messages: {
      unexpected: "This value is stringified through its base Object#toString() representation.",
    },
  },
  (context) => ({
    BinaryExpression(node: any) {
      if (node.operator !== "+") {
        return;
      }
      if (isLiteralString(node.left) || isStringLikeNode(context, node.left)) {
        reportIfUnsafe(context, node.right);
      }
      if (isLiteralString(node.right) || isStringLikeNode(context, node.right)) {
        reportIfUnsafe(context, node.left);
      }
    },
    CallExpression(node: any) {
      const [firstArgument] = node.arguments;
      if (!firstArgument) {
        return;
      }
      if (isIdentifierNamed(node.callee, "String")) {
        reportIfUnsafe(context, firstArgument);
        return;
      }
      if (calleePropertyName(node) === "toString") {
        const callee = stripChainExpression(node.callee) as any;
        reportIfUnsafe(context, callee.object);
      }
    },
    TemplateLiteral(node: any) {
      for (const expression of node.expressions ?? []) {
        reportIfUnsafe(context, expression);
      }
    },
  }),
);

function reportIfUnsafe(context: any, node: any): void {
  if (!node || !isPossiblyBaseToString(context, node)) {
    return;
  }
  context.report({
    node,
    messageId: "unexpected",
  });
}

function isPossiblyBaseToString(context: any, node: any): boolean {
  const current = stripChainExpression(node) as any;
  if (
    current?.type === "ArrayExpression" ||
    current?.type === "ObjectExpression" ||
    current?.type === "ArrowFunctionExpression" ||
    current?.type === "FunctionExpression"
  ) {
    return true;
  }
  const typeTexts = typeTextsAtNode(context, node);
  if (typeTexts.length === 0) {
    return false;
  }
  return typeTexts.some((text) => splitTopLevel(text, "|").some(isUnsafeStringifiedText));
}

function isUnsafeStringifiedText(text: string): boolean {
  const current = text.trim();
  const kind = classifyTypeText(current);
  if (
    kind === "string" ||
    kind === "number" ||
    kind === "bigint" ||
    kind === "boolean" ||
    kind === "nullish" ||
    kind === "regexp"
  ) {
    return false;
  }
  if (current === "symbol") {
    return true;
  }
  if (knownSafeObjectTypes.has(current)) {
    return false;
  }
  if (
    current === "object" ||
    current === "Object" ||
    current.startsWith("{") ||
    current.endsWith("[]") ||
    current.startsWith("[") ||
    current.startsWith("Array<") ||
    current.startsWith("ReadonlyArray<") ||
    current.startsWith("Map<") ||
    current.startsWith("ReadonlyMap<") ||
    current.startsWith("Set<") ||
    current.startsWith("ReadonlySet<") ||
    current.startsWith("Record<") ||
    current.startsWith("WeakMap<") ||
    current.startsWith("WeakSet<") ||
    current.startsWith("Promise<") ||
    current.includes("=>")
  ) {
    return true;
  }
  return false;
}

function splitTopLevel(text: string, delimiter: string): readonly string[] {
  const parts: string[] = [];
  let angleDepth = 0;
  let squareDepth = 0;
  let parenDepth = 0;
  let braceDepth = 0;
  let quote: string | undefined;
  let start = 0;

  for (let index = 0; index < text.length; index += 1) {
    const char = text[index]!;
    if (quote) {
      if (char === quote) {
        quote = undefined;
      }
      continue;
    }
    switch (char) {
      case "'":
      case '"':
      case "`":
        quote = char;
        break;
      case "<":
        angleDepth += 1;
        break;
      case ">":
        angleDepth = Math.max(0, angleDepth - 1);
        break;
      case "[":
        squareDepth += 1;
        break;
      case "]":
        squareDepth = Math.max(0, squareDepth - 1);
        break;
      case "(":
        parenDepth += 1;
        break;
      case ")":
        parenDepth = Math.max(0, parenDepth - 1);
        break;
      case "{":
        braceDepth += 1;
        break;
      case "}":
        braceDepth = Math.max(0, braceDepth - 1);
        break;
      default:
        if (
          char === delimiter &&
          angleDepth === 0 &&
          squareDepth === 0 &&
          parenDepth === 0 &&
          braceDepth === 0
        ) {
          parts.push(text.slice(start, index).trim());
          start = index + 1;
        }
        break;
    }
  }

  parts.push(text.slice(start).trim());
  return parts;
}
