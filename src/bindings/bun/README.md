# Bun Binding

The Bun binding is implemented as a Bun-friendly module that re-exports:

- the native Node.js binding when Bun is running with Node-API support
- the browser-style remote client for host-based deployments

See [`index.ts`](./index.ts).
