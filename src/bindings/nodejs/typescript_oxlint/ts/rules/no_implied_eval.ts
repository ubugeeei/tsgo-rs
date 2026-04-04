import {
  isIdentifierNamed,
  isLiteralString,
  memberPropertyName,
  stripChainExpression,
} from "./ast";
import { createNativeRule } from "./rule_creator";
import { isStringLikeNode } from "./type_utils";

const impliedEvalNames = new Set(["execScript", "setInterval", "setTimeout"]);

export const noImpliedEvalRule = createNativeRule(
  "no-implied-eval",
  {
    docs: {
      description: "Disallow string-based dynamic code execution APIs.",
    },
    messages: {
      unexpected: "Do not pass a string to an implied eval API.",
    },
  },
  (context) => ({
    CallExpression(node: any) {
      const callee = stripChainExpression(node.callee);
      const calleeName =
        memberPropertyName(callee) ?? (callee?.type === "Identifier" ? callee.name : undefined);
      if (!calleeName || !impliedEvalNames.has(calleeName)) {
        return;
      }
      const [firstArgument] = node.arguments;
      if (
        firstArgument &&
        !firstArgument.type?.includes("Function") &&
        (isLiteralString(firstArgument) || isStringLikeNode(context, firstArgument))
      ) {
        context.report({ node, messageId: "unexpected" });
      }
    },
    NewExpression(node: any) {
      if (!isIdentifierNamed(node.callee, "Function")) {
        return;
      }
      if (node.arguments.some((argument: any) => isLiteralString(argument))) {
        context.report({ node, messageId: "unexpected" });
      }
    },
  }),
);
