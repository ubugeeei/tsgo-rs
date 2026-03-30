use super::{
    callbacks::{callback_flag, jsonrpc_handlers},
    driver::ClientDriver,
};
use crate::{
    Result, TsgoError,
    jsonrpc::JsonRpcConnection,
    process::{AsyncChildGuard, TsgoCommand},
};
use std::{
    io::{BufReader, BufWriter},
    sync::Arc,
};
use tsgo_rs_core::fast::{CompactString, SmallVec};

pub(super) async fn spawn_jsonrpc_stdio(
    command: &TsgoCommand,
    filesystem: Option<Arc<dyn super::ApiFileSystem>>,
) -> Result<ClientDriver> {
    let args = stdio_args(command, filesystem.as_deref(), true);
    let mut child = command.spawn_async(args.iter().map(CompactString::as_str))?;
    let stdin = child.stdin.take().ok_or(TsgoError::Closed("api stdin"))?;
    let stdout = child.stdout.take().ok_or(TsgoError::Closed("api stdout"))?;
    let handlers = filesystem.map(jsonrpc_handlers).unwrap_or_default();
    let rpc = JsonRpcConnection::spawn(BufReader::new(stdout), BufWriter::new(stdin), handlers);
    Ok(ClientDriver::JsonRpc {
        rpc,
        process: Some(Arc::new(AsyncChildGuard::new(child))),
    })
}

pub(super) fn spawn_msgpack_stdio(
    command: &TsgoCommand,
    filesystem: Option<Arc<dyn super::ApiFileSystem>>,
) -> Result<ClientDriver> {
    let args = stdio_args(command, filesystem.as_deref(), false);
    let child = command.spawn_blocking(args.iter().map(CompactString::as_str))?;
    let worker = super::msgpack_worker::MsgpackWorker::spawn(child, filesystem)?;
    Ok(ClientDriver::Msgpack {
        worker: Arc::new(worker),
    })
}

fn stdio_args(
    command: &TsgoCommand,
    filesystem: Option<&dyn super::ApiFileSystem>,
    async_mode: bool,
) -> SmallVec<[CompactString; 6]> {
    let mut args = SmallVec::<[CompactString; 6]>::new();
    args.push(CompactString::from("--api"));
    if async_mode {
        args.push(CompactString::from("--async"));
    }
    args.push(CompactString::from("--cwd"));
    args.push(CompactString::from(command.cwd().display().to_string()));
    if let Some(filesystem) = filesystem.and_then(callback_flag) {
        args.push(filesystem);
    }
    args
}
