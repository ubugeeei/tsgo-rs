use std::time::Duration;

use crate::{Result, error::TsgoError, jsonrpc::JsonRpcConnection, process::AsyncChildGuard};
use base64::{Engine as _, engine::general_purpose::STANDARD};
use serde_json::Value;
use std::sync::Arc;
use tsgo_rs_core::fast::compact_format;

use super::msgpack_worker::MsgpackWorker;

pub(crate) enum ClientDriver {
    JsonRpc {
        rpc: JsonRpcConnection,
        process: Option<Arc<AsyncChildGuard>>,
        shutdown_timeout: Duration,
    },
    Msgpack {
        worker: Arc<MsgpackWorker>,
    },
}

impl ClientDriver {
    pub(crate) async fn request_json(&self, method: &str, params: Value) -> Result<Value> {
        match self {
            Self::JsonRpc { rpc, .. } => rpc.request_value(method, params).await,
            Self::Msgpack { worker } => {
                let payload = serde_json::to_vec(&params)?;
                let response = worker.request(method, payload).await?;
                if response.is_empty() {
                    Ok(Value::Null)
                } else {
                    Ok(serde_json::from_slice(&response)?)
                }
            }
        }
    }

    pub(crate) async fn request_binary(
        &self,
        method: &str,
        params: Value,
    ) -> Result<Option<Vec<u8>>> {
        match self {
            Self::JsonRpc { rpc, .. } => {
                let value = rpc.request_value(method, params).await?;
                if value.is_null() {
                    return Ok(None);
                }
                let data = value.get("data").and_then(Value::as_str).ok_or_else(|| {
                    TsgoError::Protocol(compact_format(format_args!(
                        "missing binary data for {method}"
                    )))
                })?;
                Ok(Some(STANDARD.decode(data)?))
            }
            Self::Msgpack { worker } => {
                let payload = serde_json::to_vec(&params)?;
                let response = worker.request(method, payload).await?;
                Ok((!response.is_empty()).then_some(response))
            }
        }
    }
    pub(crate) async fn close(&self) -> Result<()> {
        match self {
            Self::JsonRpc {
                rpc,
                process,
                shutdown_timeout,
            } => {
                rpc.close().await?;
                if let Some(process) = process {
                    process.shutdown(*shutdown_timeout).await?;
                }
                Ok(())
            }
            Self::Msgpack { worker } => worker.close().await,
        }
    }
}
