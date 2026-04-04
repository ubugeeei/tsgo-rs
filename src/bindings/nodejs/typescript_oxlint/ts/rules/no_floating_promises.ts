import {
  calleePropertyName,
  memberObject,
  nearestFunctionAncestors,
  stripChainExpression,
} from "./ast";
import { createNativeRule } from "./rule_creator";
import { isPromiseLikeNode } from "./type_utils";

export const noFloatingPromisesRule = createNativeRule(
  "no-floating-promises",
  {
    docs: {
      description: "Require promises to be awaited or otherwise handled.",
    },
    hasSuggestions: true,
    messages: {
      unexpected: "Promises must be awaited, returned, or explicitly ignored with void.",
    },
  },
  (context) => ({
    ExpressionStatement(node: any) {
      const expression = stripChainExpression(node.expression);
      if (expression?.type === "UnaryExpression" && expression.operator === "void") {
        return;
      }
      if (!isPromiseLikeNode(context, expression) || isHandled(expression)) {
        return;
      }
      context.report({
        node,
        messageId: "unexpected",
        suggest: buildSuggestions(context, node),
      });
    },
  }),
);

function isHandled(node: any): boolean {
  const current = stripChainExpression(node);
  const propertyName = calleePropertyName(current);
  if (!propertyName) {
    return false;
  }
  if (propertyName === "catch") {
    return current.arguments.length > 0;
  }
  if (propertyName === "then") {
    return current.arguments.length > 1;
  }
  if (propertyName === "finally") {
    return isHandled(memberObject(current.callee));
  }
  return false;
}

function buildSuggestions(context: any, node: any) {
  const suggestions = [
    {
      desc: "Prefix the expression with void.",
      fix: (fixer: any) => fixer.insertTextBefore(node.expression, "void "),
    },
  ];
  const nearestFunction = nearestFunctionAncestors(node, context.sourceCode)[0];
  if (nearestFunction?.async) {
    suggestions.push({
      desc: "Await the promise.",
      fix: (fixer: any) => fixer.insertTextBefore(node.expression, "await "),
    });
  }
  return suggestions;
}
