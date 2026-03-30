use crate::{
    Result, TsgoError,
    jsonrpc::{InboundEvent, JsonRpcConnection, RequestId},
    process::{AsyncChildGuard, TsgoCommand},
};
use lsp_types::{notification::Notification, request::Request};
use serde::{Serialize, de::DeserializeOwned};
use std::{io::BufReader, path::PathBuf, sync::Arc};
use tsgo_rs_core::fast::{CompactString, SmallVec};
use tsgo_rs_runtime::BroadcastReceiver;

use super::{
    InitializeApiSessionParams, InitializeApiSessionRequest, InitializeApiSessionResult, LspOverlay,
};

/// LSP client backed by the tsgo stdio server.
#[derive(Clone)]
pub struct LspClient {
    rpc: JsonRpcConnection,
    process: Arc<AsyncChildGuard>,
}

/// Spawn-time options for [`LspClient`].
///
/// # Examples
///
/// ```
/// use tsgo_rs_lsp::LspSpawnConfig;
///
/// let config = LspSpawnConfig::new("/opt/bin/tsgo")
///     .with_cwd("/workspace")
///     .with_arg("--logToFile");
///
/// assert_eq!(config.extra_args.as_slice(), &["--logToFile"]);
/// ```
#[derive(Clone, Debug)]
pub struct LspSpawnConfig {
    pub command: TsgoCommand,
    pub extra_args: SmallVec<[CompactString; 4]>,
}

impl LspSpawnConfig {
    /// Creates a new LSP spawn configuration.
    pub fn new(executable: impl Into<PathBuf>) -> Self {
        Self {
            command: TsgoCommand::new(executable),
            extra_args: SmallVec::new(),
        }
    }

    /// Sets the server working directory.
    pub fn with_cwd(mut self, cwd: impl Into<PathBuf>) -> Self {
        self.command = self.command.clone().with_cwd(cwd);
        self
    }

    /// Appends an extra CLI argument passed to tsgo.
    pub fn with_arg(mut self, arg: impl Into<CompactString>) -> Self {
        self.extra_args.push(arg.into());
        self
    }
}

impl LspClient {
    /// Spawns a tsgo LSP server over stdio.
    pub async fn spawn(config: LspSpawnConfig) -> Result<Self> {
        let mut args = SmallVec::<[CompactString; 6]>::new();
        args.push(CompactString::from("--lsp"));
        args.push(CompactString::from("--stdio"));
        args.extend(config.extra_args);
        let mut child = config
            .command
            .spawn_async(args.iter().map(CompactString::as_str))?;
        let stdin = child.stdin.take().ok_or(TsgoError::Closed("lsp stdin"))?;
        let stdout = child.stdout.take().ok_or(TsgoError::Closed("lsp stdout"))?;
        Ok(Self {
            rpc: JsonRpcConnection::spawn(BufReader::new(stdout), stdin, Default::default()),
            process: Arc::new(AsyncChildGuard::new(child)),
        })
    }

    /// Subscribes to inbound requests and notifications from the server.
    pub fn subscribe(&self) -> BroadcastReceiver<InboundEvent> {
        self.rpc.subscribe()
    }

    /// Creates a virtual-document overlay synchronized with this client.
    pub fn overlay(&self) -> LspOverlay {
        LspOverlay::new(self.clone())
    }

    /// Sends a typed LSP request.
    pub async fn request<R>(&self, params: R::Params) -> Result<R::Result>
    where
        R: Request,
        R::Params: Serialize,
        R::Result: DeserializeOwned,
    {
        self.rpc.request(R::METHOD, params).await
    }

    /// Sends a typed LSP notification.
    pub fn notify<N>(&self, params: N::Params) -> Result<()>
    where
        N: Notification,
        N::Params: Serialize,
    {
        self.rpc.notify(N::METHOD, params)
    }

    /// Responds to an inbound request.
    pub fn respond<ResultBody>(&self, id: RequestId, body: ResultBody) -> Result<()>
    where
        ResultBody: Serialize,
    {
        self.rpc.respond(id, body)
    }

    /// Calls the custom `initializeAPISession` request exposed by tsgo.
    pub async fn initialize_api_session(
        &self,
        params: InitializeApiSessionParams,
    ) -> Result<InitializeApiSessionResult> {
        self.request::<InitializeApiSessionRequest>(params).await
    }

    /// Closes the LSP transport and terminates the server process.
    pub async fn close(&self) -> Result<()> {
        self.rpc.close().await?;
        self.process
            .shutdown(std::time::Duration::from_millis(500))
            .await
    }
}
