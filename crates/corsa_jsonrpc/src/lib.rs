//! JSON-RPC framing and transport utilities used by the `tsgo` integrations.
//!
//! The types in this crate are transport-focused rather than compiler-focused:
//! they help you read and write JSON-RPC 2.0 messages over stdio or sockets,
//! route locally handled callbacks, and surface unmatched inbound requests to
//! callers.
//!
//! Reach for [`JsonRpcConnection`] when you need a reusable, thread-backed
//! request/response channel, or for [`RawMessage`] and [`RequestId`] when you
//! need to work at the protocol boundary directly.

pub mod observability {
    pub use tsgo_rs_core::{SharedObserver, TsgoEvent, TsgoObserver};
}

pub use tsgo_rs_core::{
    Result, RpcResponseError, SharedObserver, TsgoError, TsgoEvent, TsgoObserver,
};

#[path = "jsonrpc/mod.rs"]
/// JSON-RPC 2.0 message and connection primitives.
pub mod jsonrpc;

pub use jsonrpc::*;
