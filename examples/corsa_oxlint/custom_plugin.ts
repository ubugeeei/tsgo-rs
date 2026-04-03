import { definePlugin } from "corsa-oxlint";

import { noStringPlusNumberRule } from "./custom_rule.ts";

export const corsaOxlintCustomPlugin = definePlugin({
  meta: {
    name: "corsa-bind-example-plugin",
  },
  rules: {
    "no-string-plus-number": noStringPlusNumberRule,
  },
});

export default corsaOxlintCustomPlugin;
