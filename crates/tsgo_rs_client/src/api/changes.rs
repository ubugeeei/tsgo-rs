use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

use super::{DocumentIdentifier, ProjectHandle, SnapshotHandle};

/// Per-file snapshot delta grouped by operation kind.
#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FileChangeSummary {
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub changed: Vec<DocumentIdentifier>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub created: Vec<DocumentIdentifier>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub deleted: Vec<DocumentIdentifier>,
}

/// Change payload accepted by `updateSnapshot`.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(untagged)]
pub enum FileChanges {
    Summary(FileChangeSummary),
    #[serde(rename_all = "camelCase")]
    InvalidateAll {
        invalidate_all: bool,
    },
}

/// Parameters passed to [`ApiClient::update_snapshot`](crate::ApiClient::update_snapshot).
///
/// # Examples
///
/// ```
/// use tsgo_rs_client::{DocumentIdentifier, FileChangeSummary, FileChanges, UpdateSnapshotParams};
///
/// let params = UpdateSnapshotParams {
///     open_project: Some("/workspace/tsconfig.json".into()),
///     file_changes: Some(FileChanges::Summary(FileChangeSummary {
///         changed: vec![DocumentIdentifier::from("/workspace/main.ts")],
///         created: Vec::new(),
///         deleted: Vec::new(),
///     })),
/// };
///
/// assert_eq!(params.open_project.as_deref(), Some("/workspace/tsconfig.json"));
/// ```
#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateSnapshotParams {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub open_project: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub file_changes: Option<FileChanges>,
}

/// Snapshot changes scoped to a single project.
#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProjectFileChanges {
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub changed_files: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub deleted_files: Vec<String>,
}

/// Project-level delta information returned by tsgo.
#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SnapshotChanges {
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub changed_projects: BTreeMap<ProjectHandle, ProjectFileChanges>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub removed_projects: Vec<ProjectHandle>,
}

/// Raw response returned by `updateSnapshot`.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateSnapshotResponse {
    pub snapshot: SnapshotHandle,
    pub projects: Vec<super::ProjectResponse>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub changes: Option<SnapshotChanges>,
}
