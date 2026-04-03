use crate::{
    RpcResponseError,
    fast::{CompactString, compact_format},
};
use std::{io, time::Duration};

/// Workspace-wide error type for process, transport, and protocol failures.
#[derive(Debug, thiserror::Error)]
pub enum TsgoError {
    /// Underlying OS or process I/O failure.
    #[error(transparent)]
    Io(#[from] io::Error),
    /// JSON serialization or deserialization failure.
    #[error(transparent)]
    Json(#[from] serde_json::Error),
    /// Base64 decoding failure for binary JSON payloads.
    #[error(transparent)]
    Base64(#[from] base64::DecodeError),
    /// Error returned by the remote JSON-RPC peer.
    #[error("rpc error {}: {}", .0.code, .0.message)]
    Rpc(RpcResponseError),
    /// Protocol-level invariant violation or user-facing contract error.
    #[error("protocol error: {0}")]
    Protocol(CompactString),
    /// Message shape did not match what the transport expected.
    #[error("unexpected message: {0}")]
    UnexpectedMessage(CompactString),
    /// Opaque handle payload could not be parsed.
    #[error("invalid handle: {0}")]
    InvalidHandle(CompactString),
    /// Operation could not continue because the underlying resource is closed.
    #[error("process is closed: {0}")]
    Closed(&'static str),
    /// Requested feature or transport is not supported.
    #[error("unsupported: {0}")]
    Unsupported(&'static str),
    /// Thread/task join failure surfaced as a stable string.
    #[error("join error: {0}")]
    Join(CompactString),
    /// Operation did not finish before the configured deadline.
    #[error("timeout: {0}")]
    Timeout(CompactString),
}

/// Standard result alias used across the workspace.
pub type Result<T, E = TsgoError> = std::result::Result<T, E>;

impl TsgoError {
    /// Clones an error into a form safe to send to pending waiters.
    ///
    /// Some inner error types are not cheaply cloneable, so this method
    /// preserves the important semantics while normalizing them into owned
    /// variants.
    pub fn clone_for_pending(&self) -> Self {
        match self {
            Self::Io(err) => Self::Io(std::io::Error::new(
                err.kind(),
                compact_format(format_args!("{err}")).to_string(),
            )),
            Self::Json(err) => Self::Protocol(compact_format(format_args!("{err}"))),
            Self::Base64(err) => Self::Protocol(compact_format(format_args!("{err}"))),
            Self::Rpc(err) => Self::Rpc(err.clone()),
            Self::Protocol(err) => Self::Protocol(err.clone()),
            Self::UnexpectedMessage(err) => Self::UnexpectedMessage(err.clone()),
            Self::InvalidHandle(err) => Self::InvalidHandle(err.clone()),
            Self::Closed(err) => Self::Closed(err),
            Self::Unsupported(err) => Self::Unsupported(err),
            Self::Join(err) => Self::Join(err.clone()),
            Self::Timeout(err) => Self::Timeout(err.clone()),
        }
    }

    /// Creates a timeout error for a named operation.
    pub fn timeout(operation: &str, duration: Duration) -> Self {
        Self::Timeout(compact_format(format_args!(
            "{operation} timed out after {} ms",
            duration.as_millis()
        )))
    }
}
