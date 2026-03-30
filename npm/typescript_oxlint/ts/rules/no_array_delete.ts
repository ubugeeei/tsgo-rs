import { createNativeRule } from "./rule_creator";
import { isArrayLikeNode } from "./type_utils";

export const noArrayDeleteRule = createNativeRule(
  "no-array-delete",
  {
    docs: {
      description: "Disallow deleting elements from array-like values.",
    },
    hasSuggestions: true,
    messages: {
      unexpected: "Do not delete elements from an array-like value.",
      useSplice: "Use array.splice(index, 1) instead.",
    },
  },
  (context) => ({
    UnaryExpression(node: any) {
      if (
        node.operator !== "delete" ||
        node.argument?.type !== "MemberExpression" ||
        !node.argument.computed
      ) {
        return;
      }
      if (isArrayLikeNode(context, node.argument.object)) {
        context.report({
          node,
          messageId: "unexpected",
          suggest: [
            {
              messageId: "useSplice",
              fix: (fixer: any) => [
                fixer.removeRange([node.range[0], node.argument.object.range[1]]),
                fixer.replaceTextRange(
                  [node.argument.object.range[1], node.argument.property.range[0]],
                  ".splice(",
                ),
                fixer.replaceTextRange(
                  [node.argument.property.range[1], node.argument.range[1]],
                  ", 1)",
                ),
              ],
            },
          ],
        });
      }
    },
  }),
);
