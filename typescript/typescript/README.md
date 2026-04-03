# TypeScript Shared Layer

Shared TypeScript helpers for the runtime-specific bindings.

This directory holds the transport-neutral pieces that other surfaces build on:

- request/response types shared with the native Node.js wrapper
- the async remote client facade
- fetch-based transport helpers for browser-style hosts

Runtime entrypoints live alongside it:

- [`../nodejs/index.ts`](../nodejs/index.ts)
- [`../bun/index.ts`](../bun/index.ts)
- [`../deno/mod.ts`](../deno/mod.ts)
- [`../browser/index.ts`](../browser/index.ts)
