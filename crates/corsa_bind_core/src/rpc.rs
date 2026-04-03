use crate::fast::CompactString;
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct RpcResponseError {
    pub code: i64,
    pub message: CompactString,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<Value>,
}
