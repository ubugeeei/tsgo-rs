import { isIdentifierNamed, memberObject, memberPropertyName, stripChainExpression } from "./ast";
import { createNativeRule } from "./rule_creator";
import { isPromiseLikeNode } from "./type_utils";

export const awaitThenableRule = createNativeRule(
  "await-thenable",
  {
    docs: {
      description: "Disallow awaiting non-thenable values.",
    },
    messages: {
      unexpected: "Unexpected await of a non-thenable value.",
    },
  },
  (context) => ({
    AwaitExpression(node: any) {
      if (!isPromiseLikeNode(context, node.argument) && !isObviouslyPromiseLike(node.argument)) {
        context.report({ node, messageId: "unexpected" });
      }
    },
  }),
);

function isObviouslyPromiseLike(node: any): boolean {
  const current = stripChainExpression(node);
  if (current?.type === "NewExpression" && isIdentifierNamed(current.callee, "Promise")) {
    return true;
  }
  if (current?.type !== "CallExpression") {
    return false;
  }
  return (
    memberPropertyName(current.callee) === "resolve" &&
    isIdentifierNamed(memberObject(current.callee), "Promise")
  );
}
