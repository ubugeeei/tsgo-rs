use lsp_types::request::Request;
use serde::{Deserialize, Serialize};
use tsgo_rs_core::fast::CompactString;

/// Parameters for tsgo's custom `initializeAPISession` request.
#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct InitializeApiSessionParams {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pipe: Option<CompactString>,
}

/// Result returned by tsgo's custom `initializeAPISession` request.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct InitializeApiSessionResult {
    pub session_id: CompactString,
    pub pipe: CompactString,
}

/// Marker type for the custom `initializeAPISession` request.
pub enum InitializeApiSessionRequest {}

impl Request for InitializeApiSessionRequest {
    type Params = InitializeApiSessionParams;
    type Result = InitializeApiSessionResult;
    const METHOD: &'static str = "custom/initializeAPISession";
}
