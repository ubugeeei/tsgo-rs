//! `napi-rs` bindings for the `tsgo-rs` workspace.
//!
//! The module intentionally stays thin: JSON is used at the N-API boundary so
//! the Rust side can keep its typed transport and orchestration layers intact.

mod api_client;
mod document;
mod orchestrator;
mod rule_predicates;
mod util;
mod utils;

use napi_derive::napi;

/// Returns the package version exposed by the native addon.
///
/// # Examples
///
/// ```
/// assert_eq!(tsgo_rs_node::version(), env!("CARGO_PKG_VERSION"));
/// ```
#[napi]
pub fn version() -> &'static str {
    env!("CARGO_PKG_VERSION")
}
