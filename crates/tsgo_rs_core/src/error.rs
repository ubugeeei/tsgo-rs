use crate::{
    RpcResponseError,
    fast::{CompactString, compact_format},
};
use std::io;

#[derive(Debug, thiserror::Error)]
pub enum TsgoError {
    #[error(transparent)]
    Io(#[from] io::Error),
    #[error(transparent)]
    Json(#[from] serde_json::Error),
    #[error(transparent)]
    Base64(#[from] base64::DecodeError),
    #[error("rpc error {}: {}", .0.code, .0.message)]
    Rpc(RpcResponseError),
    #[error("protocol error: {0}")]
    Protocol(CompactString),
    #[error("unexpected message: {0}")]
    UnexpectedMessage(CompactString),
    #[error("invalid handle: {0}")]
    InvalidHandle(CompactString),
    #[error("process is closed: {0}")]
    Closed(&'static str),
    #[error("unsupported: {0}")]
    Unsupported(&'static str),
    #[error("join error: {0}")]
    Join(CompactString),
}

pub type Result<T, E = TsgoError> = std::result::Result<T, E>;

impl TsgoError {
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
        }
    }
}
