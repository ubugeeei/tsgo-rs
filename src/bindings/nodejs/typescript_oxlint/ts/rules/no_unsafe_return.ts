import { isUnsafeReturn } from "@corsa/node";

import { nearestFunctionAncestors } from "./ast";
import { createNativeRule } from "./rule_creator";
import { checkerFor, typeAtNode, typeTextsAtNode } from "./type_utils";

const callSignatureKind = 0;

export const noUnsafeReturnRule = createNativeRule(
  "no-unsafe-return",
  {
    docs: {
      description: "Disallow returning any-typed values from functions.",
    },
    messages: {
      unsafe: "Unsafe return of an any-typed value.",
    },
  },
  (context) => ({
    ArrowFunctionExpression(node: any) {
      if (node.body?.type === "BlockStatement") {
        return;
      }
      reportIfUnsafeReturn(context, node.body, node);
    },
    ReturnStatement(node: any) {
      if (!node.argument) {
        return;
      }
      const [owner] = nearestFunctionAncestors(node, context.sourceCode);
      if (!owner) {
        return;
      }
      reportIfUnsafeReturn(context, node.argument, owner);
    },
  }),
);

function reportIfUnsafeReturn(context: any, expression: any, owner: any): void {
  const sourceTypeTexts = typeTextsAtNode(context, expression);
  const targetTypeTexts = returnTypeTextsOfFunction(context, owner);
  if (
    !isUnsafeReturn({
      sourceTypeTexts,
      targetTypeTexts,
    })
  ) {
    return;
  }
  context.report({
    node: expression,
    messageId: "unsafe",
  });
}

function returnTypeTextsOfFunction(context: any, node: any): readonly string[] {
  const explicitAnnotation = node.returnType?.typeAnnotation ?? node.returnType;
  if (explicitAnnotation) {
    const text = context.sourceCode.getText(explicitAnnotation);
    if (text) {
      return [text];
    }
  }

  const checker = checkerFor(context);
  const type = typeAtNode(context, node);
  if (!type) {
    return [];
  }

  const texts = new Set<string>();
  for (const signature of checker.getSignaturesOfType(type, callSignatureKind)) {
    const returnType = checker.getReturnTypeOfSignature(signature);
    if (!returnType) {
      continue;
    }
    for (const text of [...(returnType.texts ?? []), checker.typeToString(returnType)]) {
      if (text) {
        texts.add(text);
      }
    }
  }

  const resolved = [...texts];
  return resolved.every(isPermissiveTypeText) ? [] : resolved;
}

function isPermissiveTypeText(text: string): boolean {
  return text === "any" || text === "unknown" || text === "never";
}
