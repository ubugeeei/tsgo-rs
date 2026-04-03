import {
  isIdentifierNamed,
  memberObject,
  memberPropertyName,
  nearestFunctionAncestors,
  stripChainExpression,
} from "./ast";
import { createNativeRule } from "./rule_creator";
import { isAnyLikeNode, isErrorLikeNode, isPromiseLikeNode, isUnknownLikeNode } from "./type_utils";

type Options = {
  allowEmptyReject?: boolean;
  allowThrowingAny?: boolean;
  allowThrowingUnknown?: boolean;
};

const defaults: Required<Options> = {
  allowEmptyReject: false,
  allowThrowingAny: false,
  allowThrowingUnknown: false,
};

export const preferPromiseRejectErrorsRule = createNativeRule(
  "prefer-promise-reject-errors",
  {
    docs: {
      description: "Require Promise rejection reasons to be Error-like values.",
    },
    messages: {
      rejectAnError: "Expected the Promise rejection reason to be an Error.",
    },
    schema: { type: "array" },
  },
  (context) => ({
    CallExpression(node: any) {
      if (!isPromiseRejectCall(context, node) && !isPromiseExecutorRejectCall(context, node)) {
        return;
      }
      if (acceptRejectArguments(context, node.arguments, resolveOptions(context.options))) {
        return;
      }
      context.report({ node, messageId: "rejectAnError" });
    },
  }),
);

function isPromiseRejectCall(context: any, node: any): boolean {
  const object = memberObject(node.callee) as any;
  return (
    memberPropertyName(node.callee) === "reject" &&
    (isIdentifierNamed(object, "Promise") || isPromiseLikeNode(context, object))
  );
}

function isPromiseExecutorRejectCall(context: any, node: any): boolean {
  const callee = stripChainExpression(node.callee);
  if (callee?.type !== "Identifier") {
    return false;
  }
  const [nearestFunction] = nearestFunctionAncestors(node, context.sourceCode);
  if (!nearestFunction) {
    return false;
  }
  const rejectParam = nearestFunction.params?.[1];
  if (!rejectParam || rejectParam.type !== "Identifier" || rejectParam.name !== callee.name) {
    return false;
  }
  const promiseConstructor = stripChainExpression(nearestFunction.parent?.parent);
  return (
    (nearestFunction.parent?.type === "NewExpression" &&
      isIdentifierNamed(nearestFunction.parent.callee, "Promise")) ||
    (promiseConstructor?.type === "NewExpression" &&
      isIdentifierNamed(promiseConstructor.callee, "Promise"))
  );
}

function acceptRejectArguments(
  context: any,
  args: readonly any[],
  options: Required<Options>,
): boolean {
  const [argument] = args;
  if (!argument) {
    return options.allowEmptyReject;
  }
  if (options.allowThrowingAny && isAnyLikeNode(context, argument)) {
    return true;
  }
  if (options.allowThrowingUnknown && isUnknownLikeNode(context, argument)) {
    return true;
  }
  return isErrorLikeNode(context, argument);
}

function resolveOptions(options: readonly unknown[]): Required<Options> {
  return { ...defaults, ...(options[0] as Options | undefined) };
}
