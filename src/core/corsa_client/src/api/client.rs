use crate::{Result, TsgoError};
use parking_lot::Mutex;
use serde::{Serialize, de::DeserializeOwned};
use serde_json::{Value, json};
use std::sync::Arc;

#[cfg(unix)]
use crate::jsonrpc::JsonRpcConnection;
#[cfg(unix)]
use std::{
    io::{BufReader, BufWriter},
    path::PathBuf,
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
/// `ApiClient` owns a single worker connection and memoizes the result of the
/// `initialize` handshake so later requests can assume the session is ready.
/// Clone values are cheap and refer to the same underlying process/transport.
///
/// # Lifecycle
///
/// 1. Create a client with [`spawn`](Self::spawn) or [`connect_pipe`](Self::connect_pipe).
/// 2. Call [`initialize`](Self::initialize) explicitly, or let endpoint helpers
///    do it lazily on first use.
/// 3. Reuse the same client for multiple snapshot and query operations.
/// 4. Call [`close`](Self::close) when the worker is no longer needed.
///
/// # Examples
///
/// ```no_run
/// use corsa_client::{ApiClient, ApiSpawnConfig};
///
/// # async fn demo() -> Result<(), corsa_client::TsgoError> {
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
    allow_unstable_upstream_calls: bool,
}

impl ApiClient {
    /// Spawns a new tsgo API worker using the supplied configuration.
    ///
    /// The underlying transport depends on [`ApiSpawnConfig::mode`]. For
    /// production and benchmark workflows, sync msgpack is typically the
    /// preferred choice because it reduces per-request overhead.
    pub async fn spawn(config: ApiSpawnConfig) -> Result<Self> {
        let driver = match config.mode {
            ApiMode::AsyncJsonRpcStdio => {
                let driver = spawn_jsonrpc_stdio(
                    &config.command,
                    config.filesystem.clone(),
                    config.request_timeout,
                    config.shutdown_timeout,
                    config.outbound_capacity,
                    config.observer.clone(),
                )
                .await?;
                Arc::new(driver)
            }
            ApiMode::SyncMsgpackStdio => {
                let driver = spawn_msgpack_stdio(
                    &config.command,
                    config.filesystem.clone(),
                    config.request_timeout,
                    config.outbound_capacity,
                    config.observer.clone(),
                )?;
                Arc::new(driver)
            }
        };
        Ok(Self {
            driver,
            initialized: Arc::new(Mutex::new(None)),
            allow_unstable_upstream_calls: config.allow_unstable_upstream_calls,
        })
    }

    #[cfg(unix)]
    /// Connects to an already-running JSON-RPC socket.
    ///
    /// This is useful when another process owns the server lifecycle and this
    /// client should only attach to the transport.
    pub async fn connect_pipe(path: impl Into<PathBuf>) -> Result<Self> {
        connect_pipe_socket(path.into()).await
    }

    /// Initializes the worker and returns the cached `initialize` response.
    ///
    /// Repeated calls are cheap: only the first call performs network I/O.
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

    /// Parses a `tsconfig` file through `tsgo`.
    ///
    /// The returned [`ConfigResponse`] contains the normalized compiler options
    /// and the file set that `tsgo` resolved for that config file.
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
    ///
    /// Snapshots are the unit of reuse for project graphs inside `tsgo`. The
    /// returned [`ManagedSnapshot`] automatically releases its remote handle
    /// when dropped, but can also be released eagerly via
    /// [`ManagedSnapshot::release`](crate::ManagedSnapshot::release).
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
    ///
    /// Returns `Ok(None)` when the file does not belong to any known project in
    /// the snapshot.
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
    ///
    /// Binary endpoints avoid JSON/base64 expansion and are a good fit for
    /// large payloads such as serialized source files.
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
    ///
    /// This is idempotent. After closing, further requests return
    /// [`TsgoError::Closed`].
    pub async fn close(&self) -> Result<()> {
        self.driver.close().await
    }

    /// Returns whether unstable upstream endpoints are allowed for this client.
    pub fn allows_unstable_upstream_calls(&self) -> bool {
        self.allow_unstable_upstream_calls
    }

    /// Sends a raw JSON endpoint request after initialization.
    ///
    /// Prefer the typed helpers where available, and use this escape hatch when
    /// experimenting with new upstream endpoints.
    pub async fn raw_json_request(&self, method: &str, params: Value) -> Result<Value> {
        self.initialize().await?;
        self.driver.request_json(method, params).await
    }

    /// Sends a raw binary endpoint request after initialization.
    ///
    /// The returned payload is wrapped in [`EncodedPayload`] for zero-surprise
    /// ownership semantics.
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
        driver: Arc::new(ClientDriver::JsonRpc {
            rpc,
            process: None,
            shutdown_timeout: std::time::Duration::from_secs(2),
        }),
        initialized: Arc::new(Mutex::new(None)),
        allow_unstable_upstream_calls: false,
    })
}
