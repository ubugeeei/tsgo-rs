use serde::{Deserialize, Serialize};
use std::{fs, path::Path};
use tsgo_rs_core::{
    Result, TsgoError,
    fast::{CompactString, compact_format},
};

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct TsgoRefLock {
    pub version: u32,
    pub typescript_go: LockedRepository,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct LockedRepository {
    pub path: CompactString,
    pub repository: CompactString,
    pub commit: CompactString,
    pub tree: CompactString,
    pub committer_date: CompactString,
    pub author: CompactString,
    pub subject: CompactString,
}

impl TsgoRefLock {
    pub fn load(path: &Path) -> Result<Self> {
        let body = fs::read_to_string(path)?;
        toml::from_str(&body).map_err(|err| {
            TsgoError::Protocol(compact_format(format_args!("invalid lockfile: {err}")))
        })
    }

    pub fn save(&self, path: &Path) -> Result<()> {
        let body = toml::to_string_pretty(self).map_err(|err| {
            TsgoError::Protocol(compact_format(format_args!(
                "failed to serialize lockfile: {err}"
            )))
        })?;
        fs::write(path, body)?;
        Ok(())
    }

    pub fn root(&self) -> &LockedRepository {
        &self.typescript_go
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lockfile_roundtrips() {
        let lock = TsgoRefLock {
            version: 1,
            typescript_go: LockedRepository {
                path: "ref/typescript-go".into(),
                repository: "https://github.com/microsoft/typescript-go.git".into(),
                commit: "abc".into(),
                tree: "def".into(),
                committer_date: "2026-03-30T00:00:00Z".into(),
                author: "Example".into(),
                subject: "Pinned".into(),
            },
        };
        let encoded = toml::to_string(&lock).unwrap();
        let decoded: TsgoRefLock = toml::from_str(&encoded).unwrap();
        assert_eq!(decoded, lock);
    }
}
