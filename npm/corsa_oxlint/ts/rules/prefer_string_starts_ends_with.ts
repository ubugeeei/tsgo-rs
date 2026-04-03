import {
  calleePropertyName,
  isZeroLiteral,
  memberObject,
  memberPropertyName,
  stripChainExpression,
} from "./ast";
import { createNativeRule } from "./rule_creator";

const comparableOperators = new Set(["==", "===", "!=", "!=="]);

export const preferStringStartsEndsWithRule = createNativeRule(
  "prefer-string-starts-ends-with",
  {
    docs: {
      description: "Prefer startsWith()/endsWith() over manual string prefix/suffix checks.",
    },
    messages: {
      endsWith: "Use endsWith() instead of slicing and comparing a suffix.",
      startsWith: "Use startsWith() instead of comparing a prefix manually.",
    },
  },
  (context) => ({
    BinaryExpression(node: any) {
      if (!comparableOperators.has(node.operator)) {
        return;
      }

      const match =
        detectManualStringCheck(context.sourceCode, node.left, node.right) ??
        detectManualStringCheck(context.sourceCode, node.right, node.left);
      if (!match) {
        return;
      }

      context.report({
        node,
        messageId: match,
      });
    },
  }),
);

function detectManualStringCheck(
  sourceCode: any,
  candidate: any,
  compared: any,
): "startsWith" | "endsWith" | undefined {
  const current = stripChainExpression(candidate) as any;
  if (current?.type !== "CallExpression") {
    return undefined;
  }

  if (calleePropertyName(current) === "indexOf" && isZeroLiteral(compared)) {
    return "startsWith";
  }

  if (calleePropertyName(current) !== "slice") {
    return undefined;
  }

  const [start, end] = current.arguments;
  if (isZeroLiteral(start) && end && sameLengthTarget(end, compared, sourceCode)) {
    return "startsWith";
  }

  const suffix = negativeLengthTarget(start);
  if (!end && suffix && sameExpression(suffix, compared, sourceCode)) {
    return "endsWith";
  }

  return undefined;
}

function sameLengthTarget(lengthNode: any, compared: any, sourceCode: any): boolean {
  if (memberPropertyName(lengthNode) !== "length") {
    return false;
  }
  const target = memberObject(lengthNode);
  return target ? sameExpression(target, compared, sourceCode) : false;
}

function negativeLengthTarget(node: any): any {
  const current = stripChainExpression(node) as any;
  if (current?.type !== "UnaryExpression" || current.operator !== "-") {
    return undefined;
  }
  const target = memberObject(current.argument);
  if (!target || memberPropertyName(current.argument) !== "length") {
    return undefined;
  }
  return target;
}

function sameExpression(left: any, right: any, sourceCode: any): boolean {
  return (
    sourceCode.getText(stripChainExpression(left)) ===
    sourceCode.getText(stripChainExpression(right))
  );
}
