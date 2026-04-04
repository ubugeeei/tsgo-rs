import { calleePropertyName, isNegativeOneLiteral, isZeroLiteral } from "./ast";
import { createNativeRule } from "./rule_creator";

export const preferIncludesRule = createNativeRule(
  "prefer-includes",
  {
    docs: {
      description: "Prefer includes over indexOf/lastIndexOf comparisons.",
    },
    messages: {
      unexpected: "Use .includes() instead of comparing an index result.",
    },
  },
  (context) => ({
    BinaryExpression(node: any) {
      if (!isComparableIndexSearch(node.left) && !isComparableIndexSearch(node.right)) {
        return;
      }
      if (
        isNegativeOneLiteral(node.left) ||
        isNegativeOneLiteral(node.right) ||
        isZeroLiteral(node.left) ||
        isZeroLiteral(node.right)
      ) {
        context.report({ node, messageId: "unexpected" });
      }
    },
  }),
);

function isComparableIndexSearch(node: any): boolean {
  const propertyName = calleePropertyName(node);
  return propertyName === "indexOf" || propertyName === "lastIndexOf";
}
