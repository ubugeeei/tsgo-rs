export function isNodeOfType(
  node: { readonly type?: string } | null | undefined,
  type: string,
): boolean {
  return node?.type === type;
}

export function isIdentifier(
  node: { readonly type?: string; readonly name?: string } | null | undefined,
  name?: string,
): boolean {
  return node?.type === "Identifier" && (name === undefined || node.name === name);
}

export const ASTUtils = Object.freeze({
  isIdentifier,
  isNodeOfType,
});
