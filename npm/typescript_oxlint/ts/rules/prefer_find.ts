import { calleePropertyName, memberObject, memberPropertyName } from "./ast";
import { createNativeRule } from "./rule_creator";

export const preferFindRule = createNativeRule(
  "prefer-find",
  {
    docs: {
      description: "Prefer find over filtering and taking the first element.",
    },
    messages: {
      unexpected: "Use .find() instead of filtering and taking the first match.",
    },
  },
  (context) => ({
    MemberExpression(node: any) {
      if (memberPropertyName(node) === "0" && calleePropertyName(node.object) === "filter") {
        context.report({ node, messageId: "unexpected" });
      }
    },
    CallExpression(node: any) {
      if (calleePropertyName(node) !== "at" || node.arguments[0]?.value !== 0) {
        return;
      }
      if (calleePropertyName(memberObject(node.callee)) === "filter") {
        context.report({ node, messageId: "unexpected" });
      }
    },
  }),
);
