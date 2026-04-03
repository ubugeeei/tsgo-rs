import { isUnsafeAssignment } from "@corsa-bind/node";

import { createNativeRule } from "./rule_creator";
import { typeTextsAtNode } from "./type_utils";

export const noUnsafeAssignmentRule = createNativeRule(
  "no-unsafe-assignment",
  {
    docs: {
      description: "Disallow assigning any-typed values to more specific targets.",
    },
    messages: {
      unsafe: "Unsafe assignment of an any-typed value.",
    },
  },
  (context) => ({
    AssignmentExpression(node: any) {
      if (node.operator !== "=") {
        return;
      }
      reportIfUnsafe(context, node.right, typeTextsAtNode(context, node.left), node);
    },
    PropertyDefinition(node: any) {
      if (!node.value) {
        return;
      }
      reportIfUnsafe(context, node.value, targetTypeTextsForNode(context, node), node);
    },
    VariableDeclarator(node: any) {
      if (!node.init) {
        return;
      }
      reportIfUnsafe(context, node.init, targetTypeTextsForNode(context, node.id), node);
    },
  }),
);

function reportIfUnsafe(
  context: any,
  sourceNode: any,
  targetTypeTexts: readonly string[],
  reportNode: any,
): void {
  const sourceTypeTexts = typeTextsAtNode(context, sourceNode);
  if (
    !isUnsafeAssignment({
      sourceTypeTexts,
      targetTypeTexts,
    })
  ) {
    return;
  }
  context.report({
    node: reportNode,
    messageId: "unsafe",
  });
}

function targetTypeTextsForNode(context: any, node: any): readonly string[] {
  const annotation = node?.typeAnnotation?.typeAnnotation ?? node?.typeAnnotation;
  if (!annotation) {
    return [];
  }
  const text = context.sourceCode.getText(annotation);
  return text ? [text] : [];
}
