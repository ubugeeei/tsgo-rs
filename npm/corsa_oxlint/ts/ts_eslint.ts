export const TSESLint = Object.freeze({});

export const tseslint = Object.freeze({
  config(...configs: readonly unknown[]) {
    return configs.flat();
  },
  configs: Object.freeze({}),
  parser: Object.freeze({
    meta: {
      name: "corsa-oxlint/parser",
      version: "0.1.0",
    },
    parseForESLint() {
      throw new Error(
        "corsa-oxlint relies on oxlint for parsing; use it as a JS plugin, not as an ESLint parser",
      );
    },
  }),
  plugin: Object.freeze({
    configs: Object.freeze({}),
    rules: Object.freeze({}),
  }),
});
