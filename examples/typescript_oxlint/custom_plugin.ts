import { definePlugin } from "oxlint-plugin-typescript-go";

import { noStringPlusNumberRule } from "./custom_rule.ts";

export const typescriptOxlintCustomPlugin = definePlugin({
  meta: {
    name: "tsgo-rs-example-plugin",
  },
  rules: {
    "no-string-plus-number": noStringPlusNumberRule,
  },
});

export default typescriptOxlintCustomPlugin;
