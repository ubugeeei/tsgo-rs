import { TsgoVirtualDocument } from "@corsa-bind/node";

import { isMain } from "../shared.ts";

export function runVirtualDocumentExample() {
  const document = TsgoVirtualDocument.untitled(
    "/virtual/demo.ts",
    "typescript",
    "const value = 1;\n",
  );

  document.applyChanges([
    {
      range: {
        start: { line: 0, character: 14 },
        end: { line: 0, character: 15 },
      },
      text: "2",
    },
  ]);

  return document.state();
}

if (isMain(import.meta.url)) {
  console.log(JSON.stringify(runVirtualDocumentExample(), null, 2));
}
