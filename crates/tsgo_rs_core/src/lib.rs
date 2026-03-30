mod error;
pub mod fast;
mod process;
mod rpc;

pub use error::{Result, TsgoError};
pub use process::{AsyncChildGuard, TsgoCommand, terminate_child_process, wait_for_child_exit};
pub use rpc::RpcResponseError;
