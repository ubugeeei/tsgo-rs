//! Typed request/response layer for the `tsgo` stdio API.
//!
//! The module is organized around a few concepts:
//!
//! - configuration and spawn types such as [`ApiSpawnConfig`] and [`ApiProfile`]
//! - lifecycle primitives such as [`ApiClient`] and [`ManagedSnapshot`]
//! - strongly typed document, handle, and response models used by endpoint
//!   helpers
//! - transport-specific helpers kept internal to the module tree
//!
//! Most consumers should start with [`ApiClient::spawn`](crate::ApiClient::spawn)
//! and [`ApiClient::update_snapshot`](crate::ApiClient::update_snapshot).

mod callbacks;
mod changes;
mod client;
mod config;
mod document;
mod driver;
mod encoded;
mod handles;
mod methods_relations;
mod methods_symbols;
mod methods_types;
mod msgpack_codec;
mod msgpack_worker;
mod requests_core;
mod requests_symbols;
mod requests_types;
mod responses;
mod snapshot;
mod spawn_stdio;

/// Filesystem callback traits and helper functions used by spawned workers.
pub use callbacks::{
    ApiFileSystem, DirectoryEntries, FileSystemCapabilities, ReadFileResult, callback_flag,
    callback_names,
};
/// Snapshot update inputs and change summaries.
pub use changes::{FileChangeSummary, FileChanges, SnapshotChanges, UpdateSnapshotParams};
/// High-level `tsgo` API client.
pub use client::ApiClient;
/// Spawn-time transport and profile configuration.
pub use config::{ApiMode, ApiProfile, ApiSpawnConfig};
/// Document identifiers and byte/UTF-16 positions used by many endpoints.
pub use document::{DocumentIdentifier, DocumentPosition};
/// Binary payload wrappers and print options.
pub use encoded::{EncodedPayload, PrintNodeOptions};
/// Opaque handles returned by `tsgo`.
pub use handles::{
    NodeHandle, ProjectHandle, SignatureHandle, SnapshotHandle, SymbolHandle, TypeHandle,
};
/// Common response payloads returned by the API.
pub use responses::{
    ConfigResponse, IndexInfo, InitializeResponse, ProjectResponse, SignatureResponse,
    SymbolResponse, TypePredicateResponse, TypeResponse,
};
/// Auto-releasing snapshot wrapper.
pub use snapshot::ManagedSnapshot;
