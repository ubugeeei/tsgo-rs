use crate::{
    Result, TsgoError,
    api::{ManagedSnapshot, ProjectResponse, SnapshotChanges, SnapshotHandle},
    lsp::{VirtualChange, VirtualDocument},
};
use serde::{Deserialize, Serialize, de::DeserializeOwned};
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use tsgo_rs_core::fast::{CompactString, FastMap, SmallVec, compact_format};

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ReplicatedSnapshot {
    pub handle: SnapshotHandle,
    pub projects: SmallVec<[ProjectResponse; 4]>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub changes: Option<SnapshotChanges>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ReplicatedCacheEntry {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub expires_at_unix_ms: Option<u64>,
    pub bytes: SmallVec<[u8; 256]>,
}

#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
pub struct ReplicatedState {
    pub documents: FastMap<CompactString, VirtualDocument>,
    pub snapshots: FastMap<CompactString, ReplicatedSnapshot>,
    pub results: FastMap<CompactString, ReplicatedCacheEntry>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(tag = "kind", rename_all = "camelCase")]
#[allow(clippy::large_enum_variant)]
pub enum ReplicatedCommand {
    PutDocument {
        document: VirtualDocument,
    },
    ApplyDocumentChange {
        uri: CompactString,
        changes: SmallVec<[VirtualChange; 4]>,
    },
    RemoveDocument {
        uri: CompactString,
    },
    PutSnapshot {
        key: CompactString,
        snapshot: ReplicatedSnapshot,
    },
    RemoveSnapshot {
        key: CompactString,
    },
    PutResult {
        key: CompactString,
        entry: ReplicatedCacheEntry,
    },
    RemoveResult {
        key: CompactString,
    },
}

impl From<&ManagedSnapshot> for ReplicatedSnapshot {
    fn from(value: &ManagedSnapshot) -> Self {
        Self {
            handle: value.handle.clone(),
            projects: value.projects.iter().cloned().collect(),
            changes: value.changes.clone(),
        }
    }
}

impl ReplicatedCacheEntry {
    pub fn new(bytes: SmallVec<[u8; 256]>, ttl: Option<Duration>) -> Self {
        Self {
            expires_at_unix_ms: ttl.map(|ttl| now_unix_ms() + ttl.as_millis() as u64),
            bytes,
        }
    }

    pub fn encode<T>(value: &T, ttl: Option<Duration>) -> Result<Self>
    where
        T: Serialize,
    {
        Ok(Self::new(
            serde_json::to_vec(value)?.into_iter().collect(),
            ttl,
        ))
    }

    pub fn decode<T>(&self) -> Result<T>
    where
        T: DeserializeOwned,
    {
        Ok(serde_json::from_slice(&self.bytes)?)
    }

    pub fn is_fresh(&self) -> bool {
        self.expires_at_unix_ms
            .map(|deadline| deadline > now_unix_ms())
            .unwrap_or(true)
    }
}

impl ReplicatedState {
    pub fn apply(&mut self, command: &ReplicatedCommand) -> Result<()> {
        match command {
            ReplicatedCommand::PutDocument { document } => {
                self.documents.insert(document.key(), document.clone());
            }
            ReplicatedCommand::ApplyDocumentChange { uri, changes } => {
                let document = self.documents.get_mut(uri.as_str()).ok_or_else(|| {
                    TsgoError::Protocol(compact_format(format_args!(
                        "unknown replicated document: {uri}"
                    )))
                })?;
                document.apply_changes(changes)?;
            }
            ReplicatedCommand::RemoveDocument { uri } => {
                self.documents.remove(uri.as_str());
            }
            ReplicatedCommand::PutSnapshot { key, snapshot } => {
                self.snapshots.insert(key.clone(), snapshot.clone());
            }
            ReplicatedCommand::RemoveSnapshot { key } => {
                self.snapshots.remove(key.as_str());
            }
            ReplicatedCommand::PutResult { key, entry } => {
                self.results.insert(key.clone(), entry.clone());
            }
            ReplicatedCommand::RemoveResult { key } => {
                self.results.remove(key.as_str());
            }
        }
        Ok(())
    }

    pub fn result<T>(&self, key: &str) -> Result<Option<T>>
    where
        T: DeserializeOwned,
    {
        self.results
            .get(key)
            .filter(|entry| entry.is_fresh())
            .map(ReplicatedCacheEntry::decode)
            .transpose()
    }
}

fn now_unix_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as u64
}

#[cfg(test)]
#[path = "state_tests.rs"]
mod tests;
