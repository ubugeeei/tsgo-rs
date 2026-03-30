export type ApiMode = "jsonrpc" | "msgpack";

export interface ApiClientOptions {
  executable: string;
  cwd?: string;
  mode?: ApiMode;
}

export interface InitializeResponse {
  useCaseSensitiveFileNames: boolean;
  currentDirectory: string;
}

export interface ConfigResponse {
  options: unknown;
  fileNames: string[];
}

export interface ProjectResponse {
  id: string;
  configFileName: string;
  compilerOptions: unknown;
  rootFiles: string[];
}

export interface FileChangeSummary {
  changed?: string[];
  created?: string[];
  deleted?: string[];
}

export type FileChanges =
  | FileChangeSummary
  | {
      invalidateAll: boolean;
    };

export interface UpdateSnapshotParams {
  openProject?: string;
  fileChanges?: FileChanges;
}

export interface UpdateSnapshotResponse {
  snapshot: string;
  projects: ProjectResponse[];
  changes?: unknown;
}

export interface TypeResponse {
  id: string;
  flags: number;
  objectFlags?: number;
  value?: unknown;
  symbol?: string;
  texts: string[];
}

export interface VirtualChange {
  range?: {
    start: { line: number; character: number };
    end: { line: number; character: number };
  };
  rangeLength?: number;
  text: string;
}

export interface VirtualDocumentState {
  uri: string;
  languageId: string;
  version: number;
  text: string;
}
