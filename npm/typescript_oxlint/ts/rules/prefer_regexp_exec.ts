import { calleePropertyName, regexFlags } from "./ast";
import { createNativeRule } from "./rule_creator";

export const preferRegexpExecRule = createNativeRule(
  "prefer-regexp-exec",
  {
    docs: {
      description: "Prefer RegExp#exec over String#match for single matches.",
    },
    messages: {
      unexpected: "Use a RegExp exec() call instead of String match().",
    },
  },
  (context) => ({
    CallExpression(node: any) {
      if (calleePropertyName(node) !== "match") {
        return;
      }
      const [firstArgument] = node.arguments;
      if (!firstArgument) {
        return;
      }
      const flags = regexFlags(firstArgument);
      if (flags !== undefined && !flags.includes("g")) {
        context.report({ node, messageId: "unexpected" });
      }
    },
  }),
);
