use std::sync::atomic::{AtomicBool, Ordering};

use crate::Result;
use tsgo_rs_runtime::spawn;

use super::{
    ApiClient, DocumentIdentifier, ProjectResponse, SnapshotChanges, SnapshotHandle,
    changes::UpdateSnapshotResponse,
};

/// Live snapshot handle with automatic release-on-drop semantics.
pub struct ManagedSnapshot {
    client: ApiClient,
    released: AtomicBool,
    pub handle: SnapshotHandle,
    pub projects: Vec<ProjectResponse>,
    pub changes: Option<SnapshotChanges>,
}

impl ManagedSnapshot {
    pub(crate) fn new(client: ApiClient, response: UpdateSnapshotResponse) -> Self {
        Self {
            client,
            released: AtomicBool::new(false),
            handle: response.snapshot,
            projects: response.projects,
            changes: response.changes,
        }
    }

    /// Looks up a project by its `tsconfig` path.
    pub fn project(&self, config_file_name: &str) -> Option<&ProjectResponse> {
        self.projects
            .iter()
            .find(|project| project.config_file_name == config_file_name)
    }

    /// Delegates to [`ApiClient::get_default_project_for_file`] using this snapshot.
    pub async fn get_default_project_for_file(
        &self,
        file: impl Into<DocumentIdentifier>,
    ) -> Result<Option<ProjectResponse>> {
        self.client
            .get_default_project_for_file(self.handle.clone(), file)
            .await
    }

    /// Releases the snapshot handle if it has not already been released.
    pub async fn release(&self) -> Result<()> {
        if self.released.swap(true, Ordering::SeqCst) {
            return Ok(());
        }
        self.client.release_handle(self.handle.as_str()).await
    }
}

impl Drop for ManagedSnapshot {
    fn drop(&mut self) {
        if self.released.load(Ordering::SeqCst) {
            return;
        }
        let client = self.client.clone();
        let snapshot = self.handle.clone();
        self.released.store(true, Ordering::SeqCst);
        let _ = spawn(async move {
            let _ = client.release_handle(snapshot.as_str()).await;
        });
    }
}
