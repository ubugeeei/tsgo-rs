use std::{path::PathBuf, sync::Arc};

use super::ApiFileSystem;
use crate::process::TsgoCommand;
use tsgo_rs_core::fast::CompactString;

/// Transport mode used to talk to the tsgo API.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum ApiMode {
    /// Async JSON-RPC over stdio.
    AsyncJsonRpcStdio,
    /// Sync msgpack tuples over stdio.
    SyncMsgpackStdio,
}

/// Process configuration for spawning a tsgo API worker.
///
/// # Examples
///
/// ```
/// use tsgo_rs_client::{ApiMode, ApiSpawnConfig};
///
/// let config = ApiSpawnConfig::new("/opt/bin/tsgo")
///     .with_cwd("/workspace")
///     .with_mode(ApiMode::AsyncJsonRpcStdio);
///
/// assert_eq!(config.mode, ApiMode::AsyncJsonRpcStdio);
/// assert_eq!(config.command.cwd().to_string_lossy(), "/workspace");
/// ```
#[derive(Clone)]
pub struct ApiSpawnConfig {
    pub command: TsgoCommand,
    pub mode: ApiMode,
    pub filesystem: Option<Arc<dyn ApiFileSystem>>,
}

impl ApiSpawnConfig {
    /// Creates a new spawn config with the fastest stdio transport enabled by default.
    ///
    /// The sync msgpack transport avoids JSON framing and base64 binary payloads,
    /// which makes it the preferred mode for benchmark and production usage.
    pub fn new(executable: impl Into<PathBuf>) -> Self {
        Self {
            command: TsgoCommand::new(executable),
            mode: ApiMode::SyncMsgpackStdio,
            filesystem: None,
        }
    }

    /// Sets the worker current directory.
    pub fn with_cwd(mut self, cwd: impl Into<PathBuf>) -> Self {
        self.command = self.command.clone().with_cwd(cwd);
        self
    }

    /// Selects the transport mode used for stdio communication.
    pub fn with_mode(mut self, mode: ApiMode) -> Self {
        self.mode = mode;
        self
    }

    /// Installs filesystem callbacks that tsgo can call back into.
    pub fn with_filesystem(mut self, filesystem: Arc<dyn ApiFileSystem>) -> Self {
        self.filesystem = Some(filesystem);
        self
    }
}

/// Named API profile reused by orchestrators and caches.
///
/// # Examples
///
/// ```
/// use tsgo_rs_client::{ApiProfile, ApiSpawnConfig};
///
/// let profile = ApiProfile::new("primary", ApiSpawnConfig::new("/opt/bin/tsgo"));
/// assert_eq!(profile.id.as_str(), "primary");
/// ```
#[derive(Clone)]
pub struct ApiProfile {
    pub id: CompactString,
    pub spawn: ApiSpawnConfig,
}

impl ApiProfile {
    /// Creates a new named profile.
    pub fn new(id: impl Into<CompactString>, spawn: ApiSpawnConfig) -> Self {
        Self {
            id: id.into(),
            spawn,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{ApiMode, ApiSpawnConfig};

    #[test]
    fn new_prefers_msgpack_fast_path() {
        let config = ApiSpawnConfig::new("/opt/bin/tsgo");
        assert_eq!(config.mode, ApiMode::SyncMsgpackStdio);
    }
}
