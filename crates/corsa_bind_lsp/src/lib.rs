//! LSP-focused client helpers for `typescript-go`.
//!
//! This crate complements `corsa_bind_client` with Language Server Protocol
//! utilities:
//!
//! - [`LspClient`] talks to `tsgo --lsp --stdio`
//! - [`LspOverlay`] keeps a mirrored set of virtual documents
//! - [`VirtualDocument`] and [`VirtualChange`] model in-memory edits in LSP
//!   coordinates
//! - custom request types expose the extra protocol extensions that `tsgo`
//!   layers on top of standard LSP
//!
//! These types are most useful for editor integrations, benchmarks that mimic
//! editor workflows, and orchestration layers that need to replicate
//! virtual-document state.

/// Re-exports JSON-RPC transport primitives used by the LSP client.
pub mod jsonrpc {
    pub use corsa_bind_jsonrpc::*;
}

/// Re-exports child-process helpers used to launch `tsgo --lsp`.
pub mod process {
    pub use corsa_bind_core::{AsyncChildGuard, TsgoCommand};
}

/// Re-exports structured operational events used by the LSP configs.
pub mod observability {
    pub use corsa_bind_core::{SharedObserver, TsgoEvent, TsgoObserver};
}

pub use corsa_bind_core::{Result, SharedObserver, TsgoError, TsgoEvent, TsgoObserver};

#[path = "lsp/mod.rs"]
/// LSP client, overlay, and custom-request types.
pub mod lsp;

pub use lsp::*;
