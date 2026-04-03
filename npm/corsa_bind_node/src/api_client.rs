use std::sync::Mutex;

use corsa_bind_rs::{
    api::{
        ApiClient, ManagedSnapshot, ProjectHandle, SnapshotHandle, TypeHandle, UpdateSnapshotParams,
    },
    fast::{CompactString, FastMap},
    runtime::block_on,
};
use napi::{Result, bindgen_prelude::Buffer};
use napi_derive::napi;
use serde::Serialize;

use crate::util::{
    SpawnOptions, build_spawn_config, into_napi_error, parse_json, parse_optional_json, to_json,
};

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct SnapshotState<'a> {
    snapshot: &'a SnapshotHandle,
    projects: &'a [corsa_bind_rs::api::ProjectResponse],
    #[serde(skip_serializing_if = "Option::is_none")]
    changes: &'a Option<corsa_bind_rs::api::SnapshotChanges>,
}

/// Thin synchronous wrapper around the Rust stdio API client.
#[napi]
pub struct TsgoApiClient {
    inner: ApiClient,
    snapshots: Mutex<FastMap<CompactString, ManagedSnapshot>>,
}

#[napi]
impl TsgoApiClient {
    /// Spawns a new client from a JSON-encoded spawn config.
    #[napi(factory)]
    pub fn spawn(options_json: String) -> Result<Self> {
        let options = parse_json::<SpawnOptions>(options_json.as_str())?;
        let inner =
            block_on(ApiClient::spawn(build_spawn_config(options)?)).map_err(into_napi_error)?;
        Ok(Self {
            inner,
            snapshots: Mutex::new(FastMap::default()),
        })
    }

    /// Calls `initialize` and returns the raw JSON response.
    #[napi]
    pub fn initialize_json(&self) -> Result<String> {
        let response = block_on(self.inner.initialize()).map_err(into_napi_error)?;
        to_json(response.as_ref())
    }

    /// Parses a `tsconfig` through tsgo and returns the JSON response.
    #[napi]
    pub fn parse_config_file_json(&self, file: String) -> Result<String> {
        let response = block_on(self.inner.parse_config_file(file)).map_err(into_napi_error)?;
        to_json(&response)
    }

    /// Applies file changes and returns a serialized snapshot record.
    #[napi]
    pub fn update_snapshot_json(&self, params_json: Option<String>) -> Result<String> {
        let params = match params_json {
            Some(params_json) => parse_json::<UpdateSnapshotParams>(params_json.as_str())?,
            None => UpdateSnapshotParams::default(),
        };
        let snapshot = block_on(self.inner.update_snapshot(params)).map_err(into_napi_error)?;
        let handle = snapshot.handle.clone();
        let state = to_json(&SnapshotState {
            snapshot: &snapshot.handle,
            projects: snapshot.projects.as_slice(),
            changes: &snapshot.changes,
        })?;
        self.snapshots
            .lock()
            .map_err(into_napi_error)?
            .insert(CompactString::from(handle.as_str()), snapshot);
        Ok(state)
    }

    /// Fetches a source file through the binary endpoint.
    #[napi]
    pub fn get_source_file(
        &self,
        snapshot: String,
        project: String,
        file: String,
    ) -> Result<Option<Buffer>> {
        let payload = block_on(self.inner.get_source_file(
            SnapshotHandle::from(snapshot.as_str()),
            ProjectHandle::from(project.as_str()),
            file,
        ))
        .map_err(into_napi_error)?;
        Ok(payload.map(|payload| Buffer::from(payload.into_bytes())))
    }

    /// Resolves the intrinsic string type for a project.
    #[napi]
    pub fn get_string_type_json(&self, snapshot: String, project: String) -> Result<String> {
        let response = block_on(self.inner.get_string_type(
            SnapshotHandle::from(snapshot.as_str()),
            ProjectHandle::from(project.as_str()),
        ))
        .map_err(into_napi_error)?;
        to_json(&response)
    }

    /// Renders a type back to a string representation.
    #[napi]
    pub fn type_to_string(
        &self,
        snapshot: String,
        project: String,
        type_handle: String,
        location: Option<String>,
        flags: Option<i32>,
    ) -> Result<String> {
        block_on(self.inner.type_to_string(
            SnapshotHandle::from(snapshot.as_str()),
            ProjectHandle::from(project.as_str()),
            TypeHandle::from(type_handle.as_str()),
            location.map(|value| corsa_bind_rs::api::NodeHandle::from(value.as_str())),
            flags,
        ))
        .map_err(into_napi_error)
    }

    /// Sends an arbitrary JSON endpoint request.
    #[napi]
    pub fn call_json(&self, method: String, params_json: Option<String>) -> Result<String> {
        let params = parse_optional_json(params_json)?;
        let response = block_on(self.inner.raw_json_request(method.as_str(), params))
            .map_err(into_napi_error)?;
        to_json(&response)
    }

    /// Sends an arbitrary binary endpoint request.
    #[napi]
    pub fn call_binary(
        &self,
        method: String,
        params_json: Option<String>,
    ) -> Result<Option<Buffer>> {
        let params = parse_optional_json(params_json)?;
        let payload = block_on(self.inner.raw_binary_request(method.as_str(), params))
            .map_err(into_napi_error)?;
        Ok(payload.map(|payload| Buffer::from(payload.into_bytes())))
    }

    /// Releases a tsgo handle explicitly.
    #[napi]
    pub fn release_handle(&self, handle: String) -> Result<()> {
        if let Some(snapshot) = self
            .snapshots
            .lock()
            .map_err(into_napi_error)?
            .remove(handle.as_str())
        {
            return block_on(snapshot.release()).map_err(into_napi_error);
        }
        let params = serde_json::json!({ "handle": handle });
        let _ =
            block_on(self.inner.raw_json_request("release", params)).map_err(into_napi_error)?;
        Ok(())
    }

    /// Closes the underlying worker process.
    #[napi]
    pub fn close(&self) -> Result<()> {
        let snapshots = std::mem::take(&mut *self.snapshots.lock().map_err(into_napi_error)?);
        for (_, snapshot) in snapshots {
            block_on(snapshot.release()).map_err(into_napi_error)?;
        }
        block_on(self.inner.close()).map_err(into_napi_error)
    }
}
