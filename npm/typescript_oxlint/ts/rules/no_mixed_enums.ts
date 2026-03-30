import { createNativeRule } from "./rule_creator";
import { isNumberLikeNode, isStringLikeNode } from "./type_utils";

export const noMixedEnumsRule = createNativeRule(
  "no-mixed-enums",
  {
    docs: {
      description: "Disallow mixing string and numeric enum members.",
    },
    messages: {
      mixed: "Mixing number and string enums can be confusing.",
    },
  },
  (context) => ({
    TSEnumDeclaration(node: any) {
      const members = enumMembersOf(node);
      if (members.length === 0) {
        return;
      }
      const desiredKind = enumMemberKind(context, members[0]);
      if (desiredKind === "unknown") {
        return;
      }
      for (const member of members) {
        const currentKind = enumMemberKind(context, member);
        if (currentKind === "unknown") {
          return;
        }
        if (currentKind !== desiredKind) {
          context.report({
            node: member.initializer ?? member,
            messageId: "mixed",
          });
          return;
        }
      }
    },
  }),
);

function enumMembersOf(node: any): readonly any[] {
  return node.body?.members ?? node.members ?? [];
}

function enumMemberKind(context: any, member: any): "number" | "string" | "unknown" {
  const initializer = member.initializer;
  if (!initializer) {
    return "number";
  }
  if (initializer.type === "Literal") {
    if (typeof initializer.value === "number") {
      return "number";
    }
    if (typeof initializer.value === "string") {
      return "string";
    }
  }
  if (isStringLikeNode(context, initializer)) {
    return "string";
  }
  if (isNumberLikeNode(context, initializer)) {
    return "number";
  }
  return "unknown";
}
