#![allow(dead_code)]

use std::{
    path::{Path, PathBuf},
    sync::Arc,
};

use corsa_bind_core::fast::{CompactString, FastMap, SmallVec};
use corsa_bind_rs::{
    api::{
        ApiFileSystem, ApiMode, ApiProfile, ApiSpawnConfig, DirectoryEntries,
        FileSystemCapabilities, ReadFileResult,
    },
    lsp::LspSpawnConfig,
};

fn executable_suffix() -> &'static str {
    if cfg!(windows) { ".exe" } else { "" }
}

pub fn mock_binary() -> PathBuf {
    if let Some(path) = std::env::var_os("CARGO_BIN_EXE_mock_tsgo") {
        let path = PathBuf::from(path);
        if path.exists() {
            return path;
        }
    }
    workspace_root()
        .join("target/debug")
        .join(format!("mock_tsgo{}", executable_suffix()))
}

pub fn workspace_root() -> PathBuf {
    find_workspace_root(Path::new(env!("CARGO_MANIFEST_DIR")))
}

fn find_workspace_root(start: &Path) -> PathBuf {
    start
        .ancestors()
        .find(|candidate| {
            candidate.join("Cargo.toml").exists()
                && candidate.join("src/core").is_dir()
                && candidate.join("src/bindings").is_dir()
        })
        .unwrap_or(start)
        .to_path_buf()
}

pub fn test_cwd() -> PathBuf {
    let cwd = std::env::temp_dir().join("corsa-bind-tests");
    std::fs::create_dir_all(&cwd).unwrap();
    cwd
}

pub fn api_config(mode: ApiMode) -> ApiSpawnConfig {
    ApiSpawnConfig::new(mock_binary())
        .with_mode(mode)
        .with_cwd(test_cwd())
}

pub fn api_profile(id: &str, mode: ApiMode) -> ApiProfile {
    ApiProfile::new(id, api_config(mode))
}

pub fn lsp_config() -> LspSpawnConfig {
    LspSpawnConfig::new(mock_binary()).with_cwd(test_cwd())
}

pub fn resolved_real_tsgo_binary() -> Option<PathBuf> {
    if let Some(path) = std::env::var_os("TSGO_EXECUTABLE") {
        let path = PathBuf::from(path);
        if path.exists() {
            return Some(path);
        }
    }
    [
        workspace_root().join(".cache/tsgo"),
        workspace_root().join(".cache/tsgo.exe"),
        workspace_root().join("origin/typescript-go/.cache/tsgo"),
        workspace_root().join("origin/typescript-go/.cache/tsgo.exe"),
        workspace_root().join("origin/typescript-go/built/local/tsgo"),
        workspace_root().join("origin/typescript-go/built/local/tsgo.exe"),
    ]
    .into_iter()
    .find(|path| path.exists())
}

pub fn real_tsgo_binary() -> PathBuf {
    resolved_real_tsgo_binary().unwrap_or_else(|| {
        workspace_root().join(if cfg!(windows) {
            ".cache/tsgo.exe"
        } else {
            ".cache/tsgo"
        })
    })
}

pub fn real_dataset() -> PathBuf {
    workspace_root().join("origin/typescript-go/_packages/api/tsconfig.json")
}

pub fn real_api_config(mode: ApiMode) -> Option<ApiSpawnConfig> {
    let binary = resolved_real_tsgo_binary()?;
    let dataset = real_dataset();
    if !dataset.exists() {
        return None;
    }
    Some(
        ApiSpawnConfig::new(binary)
            .with_mode(mode)
            .with_cwd(workspace_root()),
    )
}

pub fn virtual_fs(entries: &[(&str, &str)]) -> Arc<VirtualFs> {
    let files = entries
        .iter()
        .map(|(path, content)| (CompactString::from(*path), CompactString::from(*content)))
        .collect();
    Arc::new(VirtualFs { files })
}

pub struct VirtualFs {
    files: FastMap<CompactString, CompactString>,
}

impl ApiFileSystem for VirtualFs {
    fn capabilities(&self) -> FileSystemCapabilities {
        FileSystemCapabilities {
            read_file: true,
            file_exists: true,
            directory_exists: true,
            get_accessible_entries: true,
            realpath: true,
        }
    }

    fn read_file(&self, path: &str) -> ReadFileResult {
        self.files
            .get(path)
            .cloned()
            .map(ReadFileResult::Content)
            .unwrap_or(ReadFileResult::Fallback)
    }

    fn file_exists(&self, path: &str) -> Option<bool> {
        Some(self.files.contains_key(path))
    }

    fn directory_exists(&self, path: &str) -> Option<bool> {
        Some(path == "/virtual")
    }

    fn get_accessible_entries(&self, path: &str) -> Option<DirectoryEntries> {
        (path == "/virtual").then(|| DirectoryEntries {
            files: self
                .files
                .keys()
                .filter_map(|path| path.strip_prefix("/virtual/").map(CompactString::from))
                .collect(),
            directories: SmallVec::new(),
        })
    }

    fn realpath(&self, path: &str) -> Option<CompactString> {
        Some(path.into())
    }
}
