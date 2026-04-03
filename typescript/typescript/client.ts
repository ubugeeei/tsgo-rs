import type {
  ConfigResponse,
  InitializeResponse,
  TypeResponse,
  UpdateSnapshotParams,
  UpdateSnapshotResponse,
} from "./types.ts";

export interface TsgoRemoteTransport {
  requestBinary(method: string, params?: unknown): Promise<Uint8Array | null>;
  requestJson<T>(method: string, params?: unknown): Promise<T>;
  close?(): Promise<void> | void;
}

export interface FetchTransportOptions {
  endpoint: string | URL;
  fetch?: typeof globalThis.fetch;
  headers?: Record<string, string>;
}

type JsonEnvelope<T> = { ok: true; result: T } | { ok: false; error: string };

type BinaryEnvelope = { ok: true; bytesBase64: string | null } | { ok: false; error: string };

function decodeBase64(value: string): Uint8Array {
  const atobLike = (globalThis as { atob?: (input: string) => string }).atob;
  if (atobLike) {
    const decoded = atobLike(value);
    return Uint8Array.from(decoded, (char) => char.charCodeAt(0));
  }

  const bufferLike = globalThis as {
    Buffer?: { from(input: string, encoding: string): Uint8Array };
  };
  if (bufferLike.Buffer) {
    return Uint8Array.from(bufferLike.Buffer.from(value, "base64"));
  }

  throw new Error("base64 decoding is not available in this runtime");
}

async function parseEnvelope<T>(response: Response): Promise<JsonEnvelope<T>> {
  return (await response.json()) as JsonEnvelope<T>;
}

export function createFetchTransport(options: FetchTransportOptions): TsgoRemoteTransport {
  const fetchImpl = options.fetch ?? globalThis.fetch;
  if (!fetchImpl) {
    throw new Error("fetch is not available in this runtime");
  }

  async function post<T>(method: string, params?: unknown): Promise<T> {
    const response = await fetchImpl(options.endpoint, {
      method: "POST",
      headers: {
        "content-type": "application/json",
        ...options.headers,
      },
      body: JSON.stringify({ method, params: params ?? null }),
    });
    if (!response.ok) {
      throw new Error(`remote tsgo request failed: ${response.status} ${response.statusText}`);
    }
    const envelope = await parseEnvelope<T>(response);
    if (!envelope.ok) {
      throw new Error(envelope.error);
    }
    return envelope.result;
  }

  return {
    async requestJson<T>(method: string, params?: unknown): Promise<T> {
      return await post<T>(method, params);
    },
    async requestBinary(method: string, params?: unknown): Promise<Uint8Array | null> {
      const response = await fetchImpl(options.endpoint, {
        method: "POST",
        headers: {
          "content-type": "application/json",
          ...options.headers,
        },
        body: JSON.stringify({ method, params: params ?? null, responseType: "binary" }),
      });
      if (!response.ok) {
        throw new Error(`remote tsgo request failed: ${response.status} ${response.statusText}`);
      }
      const envelope = (await response.json()) as BinaryEnvelope;
      if (!envelope.ok) {
        throw new Error(envelope.error);
      }
      return envelope.bytesBase64 ? decodeBase64(envelope.bytesBase64) : null;
    },
  };
}

export class RemoteTsgoApiClient {
  readonly #transport: TsgoRemoteTransport;

  constructor(transport: TsgoRemoteTransport) {
    this.#transport = transport;
  }

  initialize(): Promise<InitializeResponse> {
    return this.#transport.requestJson<InitializeResponse>("initialize");
  }

  parseConfigFile(file: string): Promise<ConfigResponse> {
    return this.#transport.requestJson<ConfigResponse>("parseConfigFile", { file });
  }

  updateSnapshot(params?: UpdateSnapshotParams): Promise<UpdateSnapshotResponse> {
    return this.#transport.requestJson<UpdateSnapshotResponse>("updateSnapshot", params);
  }

  getSourceFile(snapshot: string, project: string, file: string): Promise<Uint8Array | null> {
    return this.#transport.requestBinary("getSourceFile", { snapshot, project, file });
  }

  getStringType(snapshot: string, project: string): Promise<TypeResponse> {
    return this.#transport.requestJson<TypeResponse>("getStringType", { snapshot, project });
  }

  typeToString(
    snapshot: string,
    project: string,
    typeHandle: string,
    location?: string,
    flags?: number,
  ): Promise<string> {
    return this.#transport.requestJson<string>("typeToString", {
      snapshot,
      project,
      typeHandle,
      location,
      flags,
    });
  }

  callJson<T>(method: string, params?: unknown): Promise<T> {
    return this.#transport.requestJson<T>(method, params);
  }

  callBinary(method: string, params?: unknown): Promise<Uint8Array | null> {
    return this.#transport.requestBinary(method, params);
  }

  async releaseHandle(handle: string): Promise<void> {
    await this.#transport.requestJson("release", { handle });
  }

  async close(): Promise<void> {
    await this.#transport.close?.();
  }
}

export { RemoteTsgoApiClient as BrowserTsgoApiClient };
