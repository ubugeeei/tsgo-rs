import { createNativeRule } from "./rule_creator";
import { isArrayLikeNode } from "./type_utils";

export const noForInArrayRule = createNativeRule(
  "no-for-in-array",
  {
    docs: {
      description: "Disallow for-in iteration over array-like values.",
    },
    messages: {
      unexpected: "Do not iterate over an array with a for-in loop.",
    },
  },
  (context) => ({
    ForInStatement(node: any) {
      if (isArrayLikeNode(context, node.right)) {
        context.report({ node, messageId: "unexpected" });
      }
    },
  }),
);
