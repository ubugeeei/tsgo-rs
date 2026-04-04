import { createNativeRule } from "./rule_creator";
import { isErrorLikeNode } from "./type_utils";

export const onlyThrowErrorRule = createNativeRule(
  "only-throw-error",
  {
    docs: {
      description: "Require thrown values to be Error-like.",
    },
    messages: {
      unexpected: "Only Error-like values should be thrown.",
    },
  },
  (context) => ({
    ThrowStatement(node: any) {
      if (node.argument && !isErrorLikeNode(context, node.argument)) {
        context.report({ node, messageId: "unexpected" });
      }
    },
  }),
);
