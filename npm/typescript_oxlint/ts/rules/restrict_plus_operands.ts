import { createNativeRule } from "./rule_creator";
import { classifyTypeText, typeTextAtNode } from "./type_utils";

type Options = {
  allowAny?: boolean;
  allowBoolean?: boolean;
  allowNullish?: boolean;
  allowNumberAndString?: boolean;
  allowRegExp?: boolean;
  skipCompoundAssignments?: boolean;
};

const defaults: Required<Options> = {
  allowAny: true,
  allowBoolean: true,
  allowNullish: true,
  allowNumberAndString: true,
  allowRegExp: false,
  skipCompoundAssignments: false,
};

export const restrictPlusOperandsRule = createNativeRule(
  "restrict-plus-operands",
  {
    docs: {
      description: "Require plus operands to be explicitly compatible.",
    },
    messages: {
      invalid: "Operands of + must be compatible primitive values.",
      mismatched: "Operands of + operations must be of the same type.",
    },
    schema: { type: "array" },
  },
  (context) => ({
    AssignmentExpression(node: any) {
      const options = resolveOptions(context.options);
      if (options.skipCompoundAssignments || node.operator !== "+=") {
        return;
      }
      reportIfInvalid(context, node, node.left, node.right, options);
    },
    BinaryExpression(node: any) {
      if (node.operator !== "+") {
        return;
      }
      reportIfInvalid(context, node, node.left, node.right, resolveOptions(context.options));
    },
  }),
);

function reportIfInvalid(
  context: any,
  node: any,
  left: any,
  right: any,
  options: Required<Options>,
) {
  const leftKind = classifyTypeText(typeTextAtNode(context, left));
  const rightKind = classifyTypeText(typeTextAtNode(context, right));
  if (isAllowed(leftKind, rightKind, options)) {
    return;
  }
  context.report({
    node,
    messageId:
      leftKind === "other" ||
      rightKind === "other" ||
      leftKind === "unknown" ||
      rightKind === "unknown"
        ? "invalid"
        : "mismatched",
  });
}

function isAllowed(
  leftKind: ReturnType<typeof classifyTypeText>,
  rightKind: ReturnType<typeof classifyTypeText>,
  options: Required<Options>,
): boolean {
  if (leftKind === rightKind && ["bigint", "number", "string"].includes(leftKind)) {
    return true;
  }
  if (
    options.allowNumberAndString &&
    ((leftKind === "string" && rightKind === "number") ||
      (leftKind === "number" && rightKind === "string"))
  ) {
    return true;
  }
  if (leftKind === "string" && isStringCompanion(rightKind, options)) {
    return true;
  }
  if (rightKind === "string" && isStringCompanion(leftKind, options)) {
    return true;
  }
  return false;
}

function isStringCompanion(
  kind: ReturnType<typeof classifyTypeText>,
  options: Required<Options>,
): boolean {
  return (
    (kind === "any" && options.allowAny) ||
    (kind === "boolean" && options.allowBoolean) ||
    (kind === "nullish" && options.allowNullish) ||
    (kind === "regexp" && options.allowRegExp)
  );
}

function resolveOptions(options: readonly unknown[]): Required<Options> {
  return { ...defaults, ...(options[0] as Options | undefined) };
}
