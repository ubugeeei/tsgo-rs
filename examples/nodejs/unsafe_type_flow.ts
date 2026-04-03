import { isUnsafeAssignment, isUnsafeReturn } from "@corsa-bind/node";

import { isMain } from "../shared.ts";

export function runUnsafeTypeFlowExample() {
  return {
    assignmentIntoString: isUnsafeAssignment({
      sourceTypeTexts: ["Set<any>"],
      targetTypeTexts: ["Set<string>"],
    }),
    assignmentIntoUnknown: isUnsafeAssignment({
      sourceTypeTexts: ["any"],
      targetTypeTexts: ["unknown"],
    }),
    promiseReturn: isUnsafeReturn({
      sourceTypeTexts: ["Promise<any>"],
      targetTypeTexts: ["Promise<string>"],
    }),
  };
}

if (isMain(import.meta.url)) {
  console.log(JSON.stringify(runUnsafeTypeFlowExample(), null, 2));
}
