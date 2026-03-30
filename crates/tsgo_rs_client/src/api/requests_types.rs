use serde::Serialize;

use super::{NodeHandle, ProjectHandle, SnapshotHandle, TypeHandle};

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct SignatureOfTypeRequest {
    pub snapshot: SnapshotHandle,
    pub project: ProjectHandle,
    pub r#type: TypeHandle,
    pub kind: i32,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct TypeLocationRequest {
    pub snapshot: SnapshotHandle,
    pub project: ProjectHandle,
    pub location: NodeHandle,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct PrintNodeRequest {
    pub data: String,
    #[serde(skip_serializing_if = "std::ops::Not::not")]
    pub preserve_source_newlines: bool,
    #[serde(skip_serializing_if = "std::ops::Not::not")]
    pub never_ascii_escape: bool,
    #[serde(skip_serializing_if = "std::ops::Not::not")]
    pub terminate_unterminated_literals: bool,
}
