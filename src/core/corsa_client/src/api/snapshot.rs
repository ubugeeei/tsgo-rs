use std::sync::atomic::{AtomicBool, Ordering};

use crate::Result;
use corsa_runtime::spawn;

use super::{
    ApiClient, DocumentIdentifier, ProjectResponse, SnapshotChanges, SnapshotHandle,
    changes::UpdateSnapshotResponse,
};

/// Live snapshot handle with automatic release-on-drop semantics.
///
/// A managed snapshot bundles the opaque remote handle together with the
/// project list and optional change summary returned by `updateSnapshot`. When
/// the wrapper is dropped, it schedules a best-effort handle release so callers
/// do not leak server-side snapshot state accidentally.
pub struct ManagedSnapshot {
    client: ApiClient,
    released: AtomicBool,
    /// Opaque snapshot handle used by follow-up API requests.
    pub handle: SnapshotHandle,
    /// Projects visible inside the snapshot at creation time.
    pub projects: Vec<ProjectResponse>,
    /// Optional project-level delta information returned by `tsgo`.
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
    ///
    /// This is a convenience helper for the common "find the project that owns
    /// this config file" flow after snapshot creation.
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
    ///
    /// Calling this eagerly can reduce remote memory usage in long-lived
    /// processes when the snapshot is known to be dead before Rust drop runs.
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
