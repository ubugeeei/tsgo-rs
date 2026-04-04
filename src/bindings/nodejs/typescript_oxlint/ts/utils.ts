import {
  Utils as NodeUtils,
  classifyTypeText,
  isAnyLikeTypeTexts,
  isArrayLikeTypeTexts,
  isBigIntLikeTypeTexts,
  isErrorLikeTypeTexts,
  isNumberLikeTypeTexts,
  isPromiseLikeTypeTexts,
  isStringLikeTypeTexts,
  isUnknownLikeTypeTexts,
  splitTopLevelTypeText,
  splitTypeText,
} from "@corsa/node";

export type { TypeTextKind } from "@corsa/node";

export {
  classifyTypeText,
  isAnyLikeTypeTexts,
  isArrayLikeTypeTexts,
  isBigIntLikeTypeTexts,
  isErrorLikeTypeTexts,
  isNumberLikeTypeTexts,
  isPromiseLikeTypeTexts,
  isStringLikeTypeTexts,
  isUnknownLikeTypeTexts,
  splitTopLevelTypeText,
  splitTypeText,
};

export const Utils = NodeUtils;
