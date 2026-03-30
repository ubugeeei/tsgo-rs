use serde::{Deserialize, Serialize};

/// Opaque binary payload returned by binary tsgo endpoints.
///
/// # Examples
///
/// ```
/// use tsgo_rs_client::EncodedPayload;
///
/// let payload = EncodedPayload::new(vec![1, 2, 3]);
/// assert_eq!(payload.as_bytes(), &[1, 2, 3]);
/// assert_eq!(payload.clone().into_bytes(), vec![1, 2, 3]);
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct EncodedPayload(Vec<u8>);

impl EncodedPayload {
    /// Wraps raw bytes returned by tsgo.
    pub fn new(bytes: Vec<u8>) -> Self {
        Self(bytes)
    }

    /// Borrows the payload as a byte slice.
    pub fn as_bytes(&self) -> &[u8] {
        &self.0
    }

    /// Consumes the wrapper and returns the underlying bytes.
    pub fn into_bytes(self) -> Vec<u8> {
        self.0
    }
}

/// Formatting knobs accepted by the `printNode` endpoint.
#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
pub struct PrintNodeOptions {
    #[serde(skip_serializing_if = "std::ops::Not::not")]
    pub preserve_source_newlines: bool,
    #[serde(skip_serializing_if = "std::ops::Not::not")]
    pub never_ascii_escape: bool,
    #[serde(skip_serializing_if = "std::ops::Not::not")]
    pub terminate_unterminated_literals: bool,
}
