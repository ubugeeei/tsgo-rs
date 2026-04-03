import { statSync } from "node:fs";

import { type ProjectResponse, TsgoApiClient } from "@corsa-bind/node";

import type { TsgoSignature, TsgoSymbol, TsgoType, TsgoTypePredicate } from "./types";
import type { ResolvedProjectConfig, ResolvedRuntimeOptions } from "./types";

type FileCache = {
  mtimeMs: number;
  projectId: string;
  typeByPosition: Map<number, TsgoType | undefined>;
  symbolByPosition: Map<number, TsgoSymbol | undefined>;
};

export class TsgoProjectSession {
  #client?: TsgoApiClient;
  #config?: { options: unknown; fileNames: string[] };
  #snapshot?: string;
  #projects: ProjectResponse[] = [];
  #files = new Map<string, FileCache>();
  #lastRefreshMs = 0;

  constructor(
    readonly project: ResolvedProjectConfig,
    readonly runtime: ResolvedRuntimeOptions,
  ) {}

  close(): void {
    if (this.#snapshot) {
      this.#client?.releaseHandle(this.#snapshot);
      this.#snapshot = undefined;
    }
    this.#client?.close();
    this.#client = undefined;
    this.#files.clear();
  }

  getCompilerOptions(): unknown {
    return this.config().options;
  }

  getRootFileNames(): readonly string[] {
    return this.config().fileNames;
  }

  getTypeAtPosition(fileName: string, position: number): TsgoType | undefined {
    const state = this.fileState(fileName);
    if (!state.typeByPosition.has(position)) {
      state.typeByPosition.set(
        position,
        this.client().callJson("getTypeAtPosition", {
          snapshot: this.#snapshot,
          project: state.projectId,
          file: fileName,
          position,
        }),
      );
    }
    return state.typeByPosition.get(position);
  }

  getSymbolAtPosition(fileName: string, position: number): TsgoSymbol | undefined {
    const state = this.fileState(fileName);
    if (!state.symbolByPosition.has(position)) {
      state.symbolByPosition.set(
        position,
        this.client().callJson("getSymbolAtPosition", {
          snapshot: this.#snapshot,
          project: state.projectId,
          file: fileName,
          position,
        }),
      );
    }
    return state.symbolByPosition.get(position);
  }

  getTypeOfSymbol(symbol: TsgoSymbol): TsgoType | undefined {
    return this.client().callJson("getTypeOfSymbol", {
      snapshot: this.#snapshot,
      project: this.projectId(),
      symbol: symbol.id,
    });
  }

  getDeclaredTypeOfSymbol(symbol: TsgoSymbol): TsgoType | undefined {
    return this.client().callJson("getDeclaredTypeOfSymbol", {
      snapshot: this.#snapshot,
      project: this.projectId(),
      symbol: symbol.id,
    });
  }

  typeToString(type: TsgoType, flags?: number): string {
    return this.client().typeToString(this.#snapshot!, this.projectId(), type.id, undefined, flags);
  }

  getBaseTypeOfLiteralType(type: TsgoType): TsgoType | undefined {
    return this.client().callJson("getBaseTypeOfLiteralType", {
      snapshot: this.#snapshot,
      project: this.projectId(),
      type: type.id,
    });
  }

  getPropertiesOfType(type: TsgoType): readonly TsgoSymbol[] {
    return (
      this.client().callJson("getPropertiesOfType", {
        snapshot: this.#snapshot,
        project: this.projectId(),
        type: type.id,
      }) ?? []
    );
  }

  getSignaturesOfType(type: TsgoType, kind: number): readonly TsgoSignature[] {
    return this.client().callJson("getSignaturesOfType", {
      snapshot: this.#snapshot,
      project: this.projectId(),
      type: type.id,
      kind,
    });
  }

  getReturnTypeOfSignature(signature: TsgoSignature): TsgoType | undefined {
    return this.client().callJson("getReturnTypeOfSignature", {
      snapshot: this.#snapshot,
      project: this.projectId(),
      signature: signature.id,
    });
  }

  getTypePredicateOfSignature(signature: TsgoSignature): TsgoTypePredicate | undefined {
    return this.client().callJson("getTypePredicateOfSignature", {
      snapshot: this.#snapshot,
      project: this.projectId(),
      signature: signature.id,
    });
  }

  getBaseTypes(type: TsgoType): readonly TsgoType[] {
    return (
      this.client().callJson("getBaseTypes", {
        snapshot: this.#snapshot,
        project: this.projectId(),
        type: type.id,
      }) ?? []
    );
  }

  getTypeArguments(type: TsgoType): readonly TsgoType[] {
    return (
      this.client().callJson("getTypeArguments", {
        snapshot: this.#snapshot,
        project: this.projectId(),
        type: type.id,
      }) ?? []
    );
  }

  private client(): TsgoApiClient {
    if (!this.#client) {
      this.#client = TsgoApiClient.spawn({
        executable: this.runtime.executable,
        cwd: this.runtime.cwd,
        mode: this.runtime.mode,
      });
      this.#client.initialize();
    }
    return this.#client;
  }

  private config(): { options: unknown; fileNames: string[] } {
    if (!this.#config) {
      this.#config = this.client().parseConfigFile(this.project.configPath);
    }
    const config = this.#config;
    if (!config) {
      throw new Error(
        `corsa-oxlint could not parse a tsgo config for ${this.project.configPath}`,
      );
    }
    return config;
  }

  private fileState(fileName: string): FileCache {
    this.refreshIfNeeded(fileName);
    const current = this.#files.get(fileName);
    if (current) {
      return current;
    }
    const project = this.client().callJson<ProjectResponse | null>("getDefaultProjectForFile", {
      snapshot: this.#snapshot,
      file: fileName,
    });
    const state: FileCache = {
      mtimeMs: statMtimeMs(fileName),
      projectId: project?.id ?? this.projectId(),
      typeByPosition: new Map(),
      symbolByPosition: new Map(),
    };
    this.#files.set(fileName, state);
    return state;
  }

  private refreshIfNeeded(fileName: string): void {
    const now = Date.now();
    const expired = now - this.#lastRefreshMs > this.runtime.cacheLifetimeMs;
    const stale =
      !this.#snapshot || statMtimeMs(fileName) !== this.#files.get(fileName)?.mtimeMs || expired;
    if (!stale) {
      return;
    }
    const previous = this.#snapshot;
    const response = this.client().updateSnapshot(
      previous
        ? { fileChanges: { changed: [fileName] } }
        : { openProject: this.project.configPath },
    );
    this.#snapshot = response.snapshot;
    this.#projects = response.projects;
    this.#lastRefreshMs = now;
    this.#files.clear();
    if (previous && previous !== this.#snapshot) {
      this.client().releaseHandle(previous);
    }
  }

  private projectId(): string {
    const id = this.#projects[0]?.id;
    if (!id) {
      throw new Error(
        `corsa-oxlint could not resolve a tsgo project for ${this.project.filename}`,
      );
    }
    return id;
  }
}

function statMtimeMs(fileName: string): number {
  try {
    return statSync(fileName).mtimeMs;
  } catch {
    return 0;
  }
}
