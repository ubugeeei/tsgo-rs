import type { Context, Node, SourceCode } from "@oxlint/plugins";
import type { ApiMode, ConfigResponse, ProjectResponse, TypeResponse } from "@corsa-bind/node";

export interface TsgoRuntimeOptions {
  executable?: string;
  cwd?: string;
  mode?: ApiMode;
  requestTimeoutMs?: number;
  shutdownTimeoutMs?: number;
  outboundCapacity?: number;
  allowUnstableUpstreamCalls?: boolean;
  cacheLifetimeMs?: number;
}

export interface ProjectServiceOptions {
  allowDefaultProject?: string[];
  defaultProject?: string;
}

export interface TypeAwareParserOptions {
  project?: string | string[];
  projectService?: boolean | ProjectServiceOptions;
  tsconfigRootDir?: string;
  tsgo?: TsgoRuntimeOptions;
}

export interface CorsaOxlintSettings extends TypeAwareParserOptions {
  parserOptions?: TypeAwareParserOptions;
}

export interface ResolvedRuntimeOptions {
  executable: string;
  cwd: string;
  mode: ApiMode;
  cacheLifetimeMs: number;
}

export interface ResolvedProjectConfig {
  filename: string;
  rootDir: string;
  configPath: string;
  runtime: ResolvedRuntimeOptions;
}

export interface TsgoNode {
  readonly fileName: string;
  readonly pos: number;
  readonly end: number;
  readonly range: readonly [number, number];
}

export interface TsgoSymbol {
  readonly id: string;
  readonly name: string;
  readonly flags: number;
  readonly checkFlags: number;
  readonly declarations: readonly string[];
  readonly valueDeclaration?: string;
}

export interface TsgoSignature {
  readonly id: string;
  readonly flags: number;
  readonly declaration?: string;
  readonly typeParameters: readonly string[];
  readonly parameters: readonly string[];
  readonly thisParameter?: string;
  readonly target?: string;
}

export interface TsgoTypePredicate {
  readonly kind: number;
  readonly parameterIndex: number;
  readonly parameterName?: string;
  readonly type?: TsgoType;
}

export interface TsgoType extends TypeResponse {
  readonly __corsaOxlintKind: "type";
}

export interface TsgoProgramShape {
  getCompilerOptions(): unknown;
  getCurrentDirectory(): string;
  getRootFileNames(): readonly string[];
  getSourceFile(fileName?: string): {
    readonly fileName: string;
    readonly text: string;
  };
  getTypeChecker(): TsgoTypeCheckerShape;
}

export interface TsgoTypeCheckerShape {
  getTypeAtLocation(node: Node | TsgoNode): TsgoType | undefined;
  getContextualType(node: Node | TsgoNode): TsgoType | undefined;
  getSymbolAtLocation(node: Node | TsgoNode): TsgoSymbol | undefined;
  getTypeOfSymbol(symbol: TsgoSymbol): TsgoType | undefined;
  getDeclaredTypeOfSymbol(symbol: TsgoSymbol): TsgoType | undefined;
  getTypeOfSymbolAtLocation(symbol: TsgoSymbol, node: Node | TsgoNode): TsgoType | undefined;
  typeToString(type: TsgoType, enclosingDeclaration?: Node | TsgoNode, flags?: number): string;
  getBaseTypeOfLiteralType(type: TsgoType): TsgoType | undefined;
  getPropertiesOfType(type: TsgoType): readonly TsgoSymbol[];
  getSignaturesOfType(type: TsgoType, kind: number): readonly TsgoSignature[];
  getReturnTypeOfSignature(signature: TsgoSignature): TsgoType | undefined;
  getTypePredicateOfSignature(signature: TsgoSignature): TsgoTypePredicate | undefined;
  getBaseTypes(type: TsgoType): readonly TsgoType[];
  getTypeArguments(type: TsgoType): readonly TsgoType[];
}

export interface ParserServices {
  readonly program: TsgoProgramShape;
  readonly esTreeNodeToTSNodeMap: {
    get(node: Node): TsgoNode;
    has(node: Node): boolean;
  };
  readonly tsNodeToESTreeNodeMap: {
    get(node: TsgoNode): Node;
    has(node: TsgoNode): boolean;
  };
  readonly hasFullTypeInformation: boolean;
  getTypeAtLocation(node: Node): TsgoType | undefined;
  getSymbolAtLocation(node: Node): TsgoSymbol | undefined;
}

export type ParserServicesWithTypeInformation = ParserServices & {
  readonly hasFullTypeInformation: true;
};

export type ContextWithParserOptions = Context & {
  readonly filename: string;
  readonly cwd: string;
  readonly sourceCode: SourceCode;
  readonly parserOptions?: TypeAwareParserOptions;
  readonly languageOptions?: {
    readonly parserOptions?: TypeAwareParserOptions;
  };
  readonly settings?: {
    readonly corsaOxlint?: CorsaOxlintSettings;
    readonly [key: string]: unknown;
  };
  readonly parserServices?: ParserServices;
};

export interface SessionProjectState {
  readonly config: ConfigResponse;
  readonly project: ProjectResponse;
  readonly snapshot: string;
}

export type { ProjectResponse };
