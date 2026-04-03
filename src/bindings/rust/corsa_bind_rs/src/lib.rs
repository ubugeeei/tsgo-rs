//! Top-level facade crate for the `corsa-bind` workspace.
//!
//! This crate re-exports the building blocks used to talk to `typescript-go`
//! over stdio:
//!
//! - [`api`] for the tsgo API bindings
//! - [`jsonrpc`] for stdio JSON-RPC framing and transport
//! - [`lsp`] for LSP clients and virtual-document overlays
//! - [`orchestrator`] for local orchestration plus optional experimental
//!   distributed helpers
//! - [`observability`] for structured runtime events
//! - [`runtime`] for the lightweight in-house executor
//!
//! # Examples
//!
//! ```
//! use corsa_bind_rs::{
//!     jsonrpc::RequestId,
//!     lsp::{VirtualChange, VirtualDocument},
//!     runtime::block_on,
//! };
//!
//! let mut document = VirtualDocument::untitled("/demo.ts", "typescript", "const value = 1;")?;
//! document.apply_changes(&[VirtualChange::replace("const value = 2;")])?;
//! assert_eq!(document.text, "const value = 2;");
//!
//! let request_id = RequestId::integer(7);
//! assert_eq!(request_id.to_string(), "7");
//!
//! let value = block_on(async { 21 * 2 });
//! assert_eq!(value, 42);
//! # Ok::<(), corsa_bind_rs::CorsaError>(())
//! ```

/// Re-exports the tsgo stdio API bindings.
pub mod api {
    pub use corsa_bind_client::*;
}

/// Re-exports shared error types.
pub mod error {
    pub use corsa_bind_core::{CorsaError, Result, RpcResponseError};
}

/// Re-exports performance-oriented building blocks such as `CompactString`.
pub mod fast {
    pub use corsa_bind_core::fast::*;
}

/// Re-exports shared binding utilities such as unsafe-type flow predicates.
pub mod utils {
    pub use corsa_bind_core::{CorsaUtils, UnsafeTypeFlowInput};
}

/// Re-exports JSON-RPC transport primitives.
pub mod jsonrpc {
    pub use corsa_bind_jsonrpc::*;
}

/// Re-exports LSP clients, overlays, and virtual document helpers.
pub mod lsp {
    pub use corsa_bind_lsp::*;
}

/// Re-exports structured operational events used across the workspace.
pub mod observability {
    pub use corsa_bind_core::{CorsaEvent, CorsaObserver, SharedObserver};
}

/// Re-exports client orchestration and replicated-state helpers.
pub mod orchestrator {
    pub use corsa_bind_orchestrator::{
        ApiOrchestrator, ApiOrchestratorConfig, ApiOrchestratorStats,
    };
    #[cfg(feature = "experimental-distributed")]
    pub use corsa_bind_orchestrator::{
        DistributedApiOrchestrator, RaftCluster, RaftRole, ReplicatedCacheEntry, ReplicatedCommand,
        ReplicatedSnapshot, ReplicatedState,
    };
}

/// Re-exports process spawning primitives.
pub mod process {
    pub use corsa_bind_core::{AsyncChildGuard, CorsaCommand};
}

/// Re-exports the lightweight in-house runtime.
pub mod runtime {
    pub use corsa_bind_runtime::*;
}

pub use corsa_bind_core::{
    CorsaError, CorsaEvent, CorsaObserver, CorsaUtils, Result, SharedObserver, UnsafeTypeFlowInput,
};
