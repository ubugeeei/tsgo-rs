use serde::{Deserialize, Serialize};
use tsgo_rs_core::fast::CompactString;

/// File or URI identifier accepted by tsgo API endpoints.
///
/// # Examples
///
/// ```
/// use tsgo_rs_client::DocumentIdentifier;
///
/// let file = DocumentIdentifier::from("/workspace/main.ts");
/// assert_eq!(file.as_wire_value(), "/workspace/main.ts");
/// ```
#[derive(Clone, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
#[serde(untagged)]
pub enum DocumentIdentifier {
    FileName(CompactString),
    Uri { uri: CompactString },
}

impl DocumentIdentifier {
    /// Converts the identifier into the string form used on the wire.
    pub fn as_wire_value(&self) -> CompactString {
        match self {
            Self::FileName(path) => path.clone(),
            Self::Uri { uri } => uri.clone(),
        }
    }
}

impl From<&str> for DocumentIdentifier {
    fn from(value: &str) -> Self {
        Self::FileName(CompactString::from(value))
    }
}

impl From<String> for DocumentIdentifier {
    fn from(value: String) -> Self {
        Self::FileName(CompactString::from(value))
    }
}

/// A UTF-16 position inside a document.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct DocumentPosition {
    pub document: DocumentIdentifier,
    pub position: u32,
}
