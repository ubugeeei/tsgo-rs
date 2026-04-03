//! Shared primitives used across the `corsa-bind` workspace.
//!
//! This crate intentionally stays small and foundational. It contains the
//! common error type, process lifecycle helpers, and a few performance-oriented
//! collection/string aliases used throughout the higher-level crates.
//!
//! Most applications will consume this crate indirectly via `corsa_bind_client`
//! or the top-level `corsa_bind_rs` facade, but it is also useful on its own when
//! embedding `tsgo` process management in another integration.

mod error;
/// Compact string/collection aliases used to keep hot paths allocation-light.
pub mod fast;
mod observability;
mod process;
mod rpc;

pub use error::{Result, TsgoError};
pub use observability::{SharedObserver, TsgoEvent, TsgoObserver, observe};
/// Child-process guard and reusable command template for `tsgo`.
pub use process::{AsyncChildGuard, TsgoCommand, terminate_child_process, wait_for_child_exit};
pub use rpc::RpcResponseError;
