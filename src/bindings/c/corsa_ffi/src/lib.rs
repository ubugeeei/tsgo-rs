//! C ABI wrappers for the `corsa` bindings.
//!
//! This crate exposes a small `extern "C"` surface around the Rust client,
//! virtual-document, and type-text helper APIs. Functions return default or
//! null values on failure and store the last error in thread-local state for
//! `corsa_error_message_take`.

mod api_client;
mod error;
#[cfg(test)]
mod tests;
mod types;
mod utils;
mod virtual_document;
