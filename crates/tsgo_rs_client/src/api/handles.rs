use crate::{Result, TsgoError};
use serde::{Deserialize, Serialize};
use tsgo_rs_core::fast::CompactString;

macro_rules! handle_type {
    ($name:ident) => {
        /// Opaque handle returned by tsgo.
        ///
        /// Handles are lightweight string wrappers and can be passed back to
        /// follow-up requests without parsing.
        #[derive(Clone, Debug, Deserialize, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
        #[serde(transparent)]
        pub struct $name(pub CompactString);

        impl $name {
            /// Returns the raw string representation of the handle.
            pub fn as_str(&self) -> &str {
                &self.0
            }
        }

        impl From<String> for $name {
            fn from(value: String) -> Self {
                Self(CompactString::from(value))
            }
        }

        impl From<&str> for $name {
            fn from(value: &str) -> Self {
                Self(CompactString::from(value))
            }
        }
    };
}

handle_type!(SnapshotHandle);
handle_type!(ProjectHandle);
handle_type!(SymbolHandle);
handle_type!(TypeHandle);
handle_type!(SignatureHandle);
handle_type!(NodeHandle);

/// Parsed representation of a [`NodeHandle`].
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ParsedNodeHandle {
    pub pos: u32,
    pub end: u32,
    pub kind: u16,
    pub path: CompactString,
}

impl NodeHandle {
    /// Parses a node handle into offsets, syntax kind, and backing path.
    ///
    /// # Examples
    ///
    /// ```
    /// use tsgo_rs_client::NodeHandle;
    ///
    /// let parsed = NodeHandle::from("1.5.123./workspace/main.ts").parse()?;
    /// assert_eq!(parsed.pos, 1);
    /// assert_eq!(parsed.end, 5);
    /// assert_eq!(parsed.kind, 123);
    /// assert_eq!(parsed.path.as_str(), "/workspace/main.ts");
    /// # Ok::<(), tsgo_rs_client::TsgoError>(())
    /// ```
    pub fn parse(&self) -> Result<ParsedNodeHandle> {
        let mut parts = self.0.splitn(4, '.');
        let invalid = || TsgoError::InvalidHandle(self.0.clone());
        let pos = parts
            .next()
            .ok_or_else(&invalid)?
            .parse::<u32>()
            .map_err(|_| invalid())?;
        let end = parts
            .next()
            .ok_or_else(&invalid)?
            .parse::<u32>()
            .map_err(|_| invalid())?;
        let kind = parts
            .next()
            .ok_or_else(&invalid)?
            .parse::<u16>()
            .map_err(|_| invalid())?;
        let path = parts.next().ok_or_else(&invalid)?.into();
        Ok(ParsedNodeHandle {
            pos,
            end,
            kind,
            path,
        })
    }
}

#[cfg(test)]
#[path = "handles_tests.rs"]
mod tests;
