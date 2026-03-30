import { calleePropertyName, hasUnknownTypeAnnotation } from "./ast";
import { createNativeRule } from "./rule_creator";

export const useUnknownInCatchCallbackVariableRule = createNativeRule(
  "use-unknown-in-catch-callback-variable",
  {
    docs: {
      description:
        "Require Promise catch callback variables to use an explicit unknown annotation.",
    },
    messages: {
      unexpected: "Catch callback variables should be explicitly typed as unknown.",
    },
  },
  (context) => ({
    CallExpression(node: any) {
      const propertyName = calleePropertyName(node);
      const callback =
        propertyName === "catch"
          ? node.arguments[0]
          : propertyName === "then"
            ? node.arguments[1]
            : undefined;
      const parameter = callback?.params?.[0];
      if (
        callback?.type?.includes("Function") &&
        parameter &&
        !hasUnknownTypeAnnotation(parameter)
      ) {
        context.report({ node: parameter, messageId: "unexpected" });
      }
    },
  }),
);
