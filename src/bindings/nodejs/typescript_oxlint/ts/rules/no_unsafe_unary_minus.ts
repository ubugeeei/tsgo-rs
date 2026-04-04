import { createNativeRule } from "./rule_creator";
import { classifyTypeText, splitTypeText, typeTextsAtNode } from "./type_utils";

export const noUnsafeUnaryMinusRule = createNativeRule(
  "no-unsafe-unary-minus",
  {
    docs: {
      description: "Disallow unary negation on non-number and non-bigint values.",
    },
    messages: {
      unaryMinus: "Argument of unary negation should be assignable to number | bigint.",
    },
  },
  (context) => ({
    UnaryExpression(node: any) {
      if (node.operator !== "-") {
        return;
      }
      if (isSafeLiteral(node.argument)) {
        return;
      }
      const typeTexts = typeTextsAtNode(context, node.argument);
      if (
        typeTexts.length > 0 &&
        typeTexts.every((text) => {
          return splitTypeText(text).every((part) => {
            const kind = classifyTypeText(part);
            return kind === "any" || kind === "number" || kind === "bigint";
          });
        })
      ) {
        return;
      }
      context.report({ node, messageId: "unaryMinus" });
    },
  }),
);

function isSafeLiteral(node: any): boolean {
  return (
    node?.type === "Literal" && (typeof node.value === "number" || typeof node.bigint === "string")
  );
}
