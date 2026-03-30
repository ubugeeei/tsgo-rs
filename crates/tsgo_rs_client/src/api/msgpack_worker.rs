use crate::{Result, TsgoError};
use parking_lot::Mutex;
use std::{
    io::{BufReader, BufWriter},
    sync::{Arc, mpsc},
    thread,
};
use tsgo_rs_core::fast::{CompactString, compact_format};
use tsgo_rs_core::terminate_child_process;

use super::{
    callbacks::{ApiFileSystem, invoke_callback},
    msgpack_codec::{
        MSG_CALL, MSG_CALL_ERROR, MSG_CALL_RESPONSE, MSG_ERROR, MSG_REQUEST, MSG_RESPONSE,
        MsgpackTuple, read_tuple, write_tuple,
    },
};

pub(crate) struct MsgpackWorker {
    tx: mpsc::Sender<WorkerCommand>,
    join: Mutex<Option<thread::JoinHandle<()>>>,
}

pub(crate) struct WorkerResponse {
    pub bytes: Vec<u8>,
}

enum WorkerCommand {
    Request {
        method: CompactString,
        payload: Vec<u8>,
        reply: mpsc::SyncSender<Result<WorkerResponse>>,
    },
    Shutdown,
}

impl MsgpackWorker {
    pub(crate) fn spawn(
        mut child: std::process::Child,
        filesystem: Option<Arc<dyn ApiFileSystem>>,
    ) -> Result<Self> {
        let stdin = child
            .stdin
            .take()
            .ok_or(TsgoError::Closed("msgpack stdin"))?;
        let stdout = child
            .stdout
            .take()
            .ok_or(TsgoError::Closed("msgpack stdout"))?;
        let (tx, rx) = mpsc::channel::<WorkerCommand>();
        let join = thread::spawn(move || {
            let mut writer = BufWriter::new(stdin);
            let mut reader = BufReader::new(stdout);
            while let Ok(command) = rx.recv() {
                match command {
                    WorkerCommand::Request {
                        method,
                        payload,
                        reply,
                    } => {
                        let method = method.as_bytes().to_vec();
                        let result = write_tuple(&mut writer, MSG_REQUEST, &method, &payload)
                            .and_then(|_| {
                                read_response(
                                    &mut reader,
                                    &mut writer,
                                    &method,
                                    filesystem.as_deref(),
                                )
                            });
                        let _ = reply.send(result.map(|bytes| WorkerResponse { bytes }));
                    }
                    WorkerCommand::Shutdown => break,
                }
            }
            let _ = terminate_child_process(&mut child);
        });
        Ok(Self {
            tx,
            join: Mutex::new(Some(join)),
        })
    }

    pub(crate) async fn request(&self, method: &str, payload: Vec<u8>) -> Result<Vec<u8>> {
        let (reply_tx, reply_rx) = mpsc::sync_channel(1);
        self.tx
            .send(WorkerCommand::Request {
                method: CompactString::from(method),
                payload,
                reply: reply_tx,
            })
            .map_err(|_| TsgoError::Closed("msgpack worker"))?;
        Ok(reply_rx
            .recv()
            .map_err(|_| TsgoError::Closed("msgpack worker"))??
            .bytes)
    }

    pub(crate) async fn close(&self) -> Result<()> {
        let _ = self.tx.send(WorkerCommand::Shutdown);
        if let Some(join) = self.join.lock().take() {
            join.join()
                .map_err(|_| TsgoError::Join("msgpack worker".into()))?;
        }
        Ok(())
    }
}

fn read_response(
    reader: &mut BufReader<std::process::ChildStdout>,
    writer: &mut BufWriter<std::process::ChildStdin>,
    method: &[u8],
    filesystem: Option<&dyn ApiFileSystem>,
) -> Result<Vec<u8>> {
    loop {
        let message = read_tuple(reader)?;
        match message.kind {
            MSG_RESPONSE if message.method == method => return Ok(message.payload),
            MSG_ERROR if message.method == method => {
                return Err(TsgoError::Protocol(
                    String::from_utf8_lossy(&message.payload).into(),
                ));
            }
            MSG_CALL => handle_callback(writer, filesystem, message)?,
            other => {
                return Err(TsgoError::UnexpectedMessage(compact_format(format_args!(
                    "msgpack type {other}"
                ))));
            }
        }
    }
}

fn handle_callback(
    writer: &mut BufWriter<std::process::ChildStdin>,
    filesystem: Option<&dyn ApiFileSystem>,
    callback: MsgpackTuple,
) -> Result<()> {
    let method = std::str::from_utf8(&callback.method)
        .map_err(|_| TsgoError::Protocol("callback method must be utf-8".into()))?;
    let Some(filesystem) = filesystem else {
        return write_tuple(
            writer,
            MSG_CALL_ERROR,
            method.as_bytes(),
            b"no filesystem callbacks",
        );
    };
    let payload: serde_json::Value = serde_json::from_slice(&callback.payload)?;
    let value = match invoke_callback(filesystem, method, &payload) {
        Ok(value) => value,
        Err(error) => {
            return write_tuple(
                writer,
                MSG_CALL_ERROR,
                method.as_bytes(),
                error.message.as_bytes(),
            );
        }
    };
    match serde_json::to_vec(&value) {
        Ok(bytes) => write_tuple(writer, MSG_CALL_RESPONSE, method.as_bytes(), &bytes),
        Err(error) => write_tuple(
            writer,
            MSG_CALL_ERROR,
            method.as_bytes(),
            error.to_string().as_bytes(),
        ),
    }
}
