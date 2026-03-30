use crate::{Result, TsgoError, jsonrpc::JsonRpcConnection};
use parking_lot::Mutex;
use serde::{Serialize, de::DeserializeOwned};
use serde_json::{Value, json};
use std::{
    io::{BufReader, BufWriter},
    path::PathBuf,
    sync::Arc,
};

use super::{
    changes::{UpdateSnapshotParams, UpdateSnapshotResponse},
    config::{ApiMode, ApiSpawnConfig},
    document::DocumentIdentifier,
    driver::ClientDriver,
    encoded::EncodedPayload,
    requests_core::{
        ParseConfigFileRequest, ReleaseRequest, SnapshotFileRequest, UpdateSnapshotRequest,
    },
    responses::{ConfigResponse, InitializeResponse, ProjectResponse},
    snapshot::ManagedSnapshot,
    spawn_stdio::{spawn_jsonrpc_stdio, spawn_msgpack_stdio},
};

/// High-level client for the tsgo stdio API.
///
/// # Examples
///
/// ```no_run
/// use tsgo_rs_client::{ApiClient, ApiSpawnConfig};
///
/// # async fn demo() -> Result<(), tsgo_rs_client::TsgoError> {
/// let client = ApiClient::spawn(ApiSpawnConfig::new("/opt/bin/tsgo")).await?;
/// let _initialize = client.initialize().await?;
/// client.close().await?;
/// # Ok(())
/// # }
/// ```
#[derive(Clone)]
pub struct ApiClient {
    driver: Arc<ClientDriver>,
    initialized: Arc<Mutex<Option<Arc<InitializeResponse>>>>,
}

impl ApiClient {
    /// Spawns a new tsgo API worker using the supplied configuration.
    pub async fn spawn(config: ApiSpawnConfig) -> Result<Self> {
        let driver = match config.mode {
            ApiMode::AsyncJsonRpcStdio => {
                let driver =
                    spawn_jsonrpc_stdio(&config.command, config.filesystem.clone()).await?;
                Arc::new(driver)
            }
            ApiMode::SyncMsgpackStdio => {
                let driver = spawn_msgpack_stdio(&config.command, config.filesystem.clone())?;
                Arc::new(driver)
            }
        };
        Ok(Self {
            driver,
            initialized: Arc::new(Mutex::new(None)),
        })
    }

    #[cfg(unix)]
    /// Connects to an already-running JSON-RPC socket.
    pub async fn connect_pipe(path: impl Into<PathBuf>) -> Result<Self> {
        connect_pipe_socket(path.into()).await
    }

    /// Initializes the worker and returns the cached `initialize` response.
    pub async fn initialize(&self) -> Result<Arc<InitializeResponse>> {
        if self.initialized.lock().is_none() {
            let value = self.driver.request_json("initialize", Value::Null).await?;
            let init: Arc<InitializeResponse> = Arc::new(serde_json::from_value(value)?);
            let mut slot = self.initialized.lock();
            if slot.is_none() {
                *slot = Some(init.clone());
            }
        }
        self.initialized
            .lock()
            .as_ref()
            .cloned()
            .ok_or(TsgoError::Closed("api initialize"))
    }

    /// Parses a `tsconfig` file through tsgo.
    pub async fn parse_config_file(
        &self,
        file: impl Into<DocumentIdentifier>,
    ) -> Result<ConfigResponse> {
        self.initialize().await?;
        let request = ParseConfigFileRequest { file: file.into() };
        let value = self
            .driver
            .request_json("parseConfigFile", serde_json::to_value(request)?)
            .await?;
        Ok(serde_json::from_value(value)?)
    }

    /// Applies file changes and returns a managed snapshot handle.
    pub async fn update_snapshot(&self, params: UpdateSnapshotParams) -> Result<ManagedSnapshot> {
        self.initialize().await?;
        let request = UpdateSnapshotRequest {
            open_project: params.open_project,
            file_changes: params.file_changes,
        };
        let value = self
            .driver
            .request_json("updateSnapshot", serde_json::to_value(request)?)
            .await?;
        let response: UpdateSnapshotResponse = serde_json::from_value(value)?;
        Ok(super::snapshot::ManagedSnapshot::new(
            self.clone(),
            response,
        ))
    }

    /// Resolves the default project for a file inside a snapshot.
    pub async fn get_default_project_for_file(
        &self,
        snapshot: super::SnapshotHandle,
        file: impl Into<DocumentIdentifier>,
    ) -> Result<Option<ProjectResponse>> {
        let request = SnapshotFileRequest {
            snapshot,
            file: file.into(),
        };
        let value = self
            .driver
            .request_json("getDefaultProjectForFile", serde_json::to_value(request)?)
            .await?;
        if value.is_null() {
            Ok(None)
        } else {
            Ok(Some(serde_json::from_value(value)?))
        }
    }

    /// Fetches a source file via a binary endpoint.
    pub async fn get_source_file(
        &self,
        snapshot: super::SnapshotHandle,
        project: super::ProjectHandle,
        file: impl Into<DocumentIdentifier>,
    ) -> Result<Option<EncodedPayload>> {
        let request = SnapshotFileRequest {
            snapshot,
            file: file.into(),
        };
        let request =
            json!({ "snapshot": request.snapshot, "project": project, "file": request.file });
        Ok(self
            .driver
            .request_binary("getSourceFile", request)
            .await?
            .map(EncodedPayload::new))
    }

    /// Closes the client and shuts down the underlying worker process.
    pub async fn close(&self) -> Result<()> {
        self.driver.close().await
    }

    /// Sends a raw JSON endpoint request after initialization.
    pub async fn raw_json_request(&self, method: &str, params: Value) -> Result<Value> {
        self.initialize().await?;
        self.driver.request_json(method, params).await
    }

    /// Sends a raw binary endpoint request after initialization.
    pub async fn raw_binary_request(
        &self,
        method: &str,
        params: Value,
    ) -> Result<Option<EncodedPayload>> {
        self.initialize().await?;
        Ok(self
            .driver
            .request_binary(method, params)
            .await?
            .map(EncodedPayload::new))
    }

    pub(crate) async fn release_handle(&self, handle: &str) -> Result<()> {
        let request = ReleaseRequest { handle };
        let _ = self
            .driver
            .request_json("release", serde_json::to_value(request)?)
            .await?;
        Ok(())
    }

    pub(crate) async fn call<T, P>(&self, method: &str, params: P) -> Result<T>
    where
        T: DeserializeOwned,
        P: Serialize,
    {
        let value = self
            .raw_json_request(method, serde_json::to_value(params)?)
            .await?;
        Ok(serde_json::from_value(value)?)
    }

    pub(crate) async fn call_optional<T, P>(&self, method: &str, params: P) -> Result<Option<T>>
    where
        T: DeserializeOwned,
        P: Serialize,
    {
        let value = self
            .raw_json_request(method, serde_json::to_value(params)?)
            .await?;
        if value.is_null() {
            Ok(None)
        } else {
            Ok(Some(serde_json::from_value(value)?))
        }
    }

    pub(crate) async fn call_optional_binary<P>(
        &self,
        method: &str,
        params: P,
    ) -> Result<Option<EncodedPayload>>
    where
        P: Serialize,
    {
        self.raw_binary_request(method, serde_json::to_value(params)?)
            .await
    }
}

#[cfg(unix)]
async fn connect_pipe_socket(path: PathBuf) -> Result<ApiClient> {
    let stream = std::os::unix::net::UnixStream::connect(path)?;
    let reader = BufReader::new(stream.try_clone()?);
    let writer = BufWriter::new(stream);
    let rpc = JsonRpcConnection::spawn(reader, writer, Default::default());
    Ok(ApiClient {
        driver: Arc::new(ClientDriver::JsonRpc { rpc, process: None }),
        initialized: Arc::new(Mutex::new(None)),
    })
}
