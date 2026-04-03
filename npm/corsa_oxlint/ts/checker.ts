import type { Node } from "@oxlint/plugins";

import { createNodeMaps, toPosition } from "./node_map";
import { sessionForContext } from "./registry";
import type {
  ContextWithParserOptions,
  TsgoNode,
  TsgoProgramShape,
  TsgoSignature,
  TsgoSymbol,
  TsgoType,
  TsgoTypeCheckerShape,
} from "./types";

export function createProgram(
  context: ContextWithParserOptions,
): TsgoProgramShape & { readonly nodeMaps: ReturnType<typeof createNodeMaps> } {
  const nodeMaps = createNodeMaps(context);
  return {
    nodeMaps,
    getCompilerOptions() {
      return sessionForContext(context).session.getCompilerOptions();
    },
    getCurrentDirectory() {
      return sessionForContext(context).project.rootDir;
    },
    getRootFileNames() {
      return sessionForContext(context).session.getRootFileNames();
    },
    getSourceFile(fileName = context.filename) {
      return { fileName, text: context.sourceCode.text };
    },
    getTypeChecker() {
      return createTypeChecker(context);
    },
  };
}

export function createTypeChecker(context: ContextWithParserOptions): TsgoTypeCheckerShape {
  return {
    getTypeAtLocation(node) {
      return sessionForContext(context).session.getTypeAtPosition(
        filenameFor(context, node),
        toPosition(node),
      );
    },
    getContextualType(node) {
      return this.getTypeAtLocation(node);
    },
    getSymbolAtLocation(node) {
      return sessionForContext(context).session.getSymbolAtPosition(
        filenameFor(context, node),
        toPosition(node),
      );
    },
    getTypeOfSymbol(symbol) {
      return sessionForContext(context).session.getTypeOfSymbol(symbol);
    },
    getDeclaredTypeOfSymbol(symbol) {
      return sessionForContext(context).session.getDeclaredTypeOfSymbol(symbol);
    },
    getTypeOfSymbolAtLocation(symbol, node) {
      return this.getTypeAtLocation(node) ?? this.getTypeOfSymbol(symbol);
    },
    typeToString(type, enclosingDeclaration, flags) {
      void enclosingDeclaration;
      return sessionForContext(context).session.typeToString(type, flags);
    },
    getBaseTypeOfLiteralType(type) {
      return sessionForContext(context).session.getBaseTypeOfLiteralType(type);
    },
    getPropertiesOfType(type) {
      return sessionForContext(context).session.getPropertiesOfType(type);
    },
    getSignaturesOfType(type, kind) {
      return sessionForContext(context).session.getSignaturesOfType(type, kind);
    },
    getReturnTypeOfSignature(signature) {
      return sessionForContext(context).session.getReturnTypeOfSignature(signature);
    },
    getTypePredicateOfSignature(signature) {
      return sessionForContext(context).session.getTypePredicateOfSignature(signature);
    },
    getBaseTypes(type) {
      return sessionForContext(context).session.getBaseTypes(type);
    },
    getTypeArguments(type) {
      return sessionForContext(context).session.getTypeArguments(type);
    },
  };
}

function filenameFor(
  context: ContextWithParserOptions,
  node: Node | TsgoNode | TsgoType | TsgoSymbol | TsgoSignature,
): string {
  if ("fileName" in node) {
    return node.fileName;
  }
  return context.filename;
}
