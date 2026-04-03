//! High-level client bindings for the `typescript-go` stdio API.
//!
//! This crate wraps the raw transports and endpoint naming used by `tsgo`
//! behind typed request/response helpers. In practice it is the main entry
//! point when you want to:
//!
//! - spawn a `tsgo` worker process
//! - initialize it once and reuse the session
//! - create and reuse snapshots
//! - ask type, symbol, and syntax questions through strongly typed helpers
//! - attach filesystem callbacks for overlay-like workflows
//!
//! # Main Building Blocks
//!
//! - [`ApiClient`] manages a single worker process or pipe connection.
//! - [`ApiSpawnConfig`] describes how that worker should be started.
//! - [`ManagedSnapshot`] keeps snapshot handles alive and releases them on drop.
//! - [`ApiProfile`] gives orchestrators a stable name for a spawn configuration.
//!
//! # Performance Model
//!
//! `corsa-bind` does not try to out-compile `tsgo` itself. The win comes from
//! session reuse, snapshot reuse, and cheaper transports such as sync msgpack.
//! For docs and benchmarks around that trade-off, see the workspace guides.

/// Re-exports shared error types used by the client APIs.
pub mod error {
    pub use corsa_bind_core::{CorsaError, Result, RpcResponseError};
}

/// Re-exports low-level JSON-RPC helpers used by the stdio client transport.
pub mod jsonrpc {
    pub use corsa_bind_jsonrpc::*;
}

/// Re-exports process-spawning primitives used to launch `tsgo`.
pub mod process {
    pub use corsa_bind_core::{AsyncChildGuard, CorsaCommand};
}

/// Re-exports structured operational events used by the client configs.
pub mod observability {
    pub use corsa_bind_core::{CorsaEvent, CorsaObserver, SharedObserver};
}

pub use corsa_bind_core::{CorsaError, CorsaEvent, CorsaObserver, Result, SharedObserver};

#[path = "api/mod.rs"]
/// Typed bindings for the `tsgo` stdio API surface.
pub mod api;

pub use api::*;
