use crate::{Result, TsgoError};
use parking_lot::Mutex;
use serde::{Serialize, de::DeserializeOwned};
use serde_json::Value;
use std::{
    io::{BufRead, Write},
    sync::mpsc,
    sync::{
        Arc,
        atomic::{AtomicBool, AtomicI64, Ordering},
    },
    thread,
};
use tsgo_rs_core::fast::{CompactString, FastMap};
use tsgo_rs_runtime::{BroadcastReceiver, BroadcastSender, broadcast};

use super::{
    RequestId,
    frame::{read_frame, write_frame},
    message::{MessageKind, RawMessage, RpcResponseError},
};

pub type RpcHandler =
    Arc<dyn Fn(Value) -> std::result::Result<Value, RpcResponseError> + Send + Sync + 'static>;
/// Handler map keyed by JSON-RPC method name.
pub type RpcHandlerMap = FastMap<CompactString, RpcHandler>;

/// Inbound JSON-RPC events that are not handled locally.
#[derive(Clone, Debug)]
pub enum InboundEvent {
    Request {
        id: RequestId,
        method: CompactString,
        params: Value,
    },
    Notification {
        method: CompactString,
        params: Value,
    },
}

/// Thread-backed stdio JSON-RPC connection.
///
/// The connection owns a reader thread, a writer thread, and a pending-request
/// table. Requests and notifications are serialized on the writer thread, while
/// inbound frames are decoded on the reader thread.
#[derive(Clone)]
pub struct JsonRpcConnection {
    inner: Arc<Inner>,
}

struct Inner {
    closed: AtomicBool,
    next_id: AtomicI64,
    events: BroadcastSender<InboundEvent>,
    handlers: RpcHandlerMap,
    outbound: Mutex<Option<mpsc::Sender<RawMessage>>>,
    pending: Mutex<FastMap<RequestId, mpsc::SyncSender<Result<Value>>>>,
    read_task: Mutex<Option<thread::JoinHandle<()>>>,
    write_task: Mutex<Option<thread::JoinHandle<()>>>,
}

impl JsonRpcConnection {
    /// Spawns a connection around a buffered reader and writer.
    pub fn spawn<R, W>(reader: R, writer: W, handlers: RpcHandlerMap) -> Self
    where
        R: BufRead + Send + 'static,
        W: Write + Send + 'static,
    {
        let (outbound_tx, outbound_rx) = mpsc::channel();
        let (events, _) = broadcast();
        let inner = Arc::new(Inner {
            closed: AtomicBool::new(false),
            next_id: AtomicI64::new(0),
            events,
            handlers,
            outbound: Mutex::new(Some(outbound_tx)),
            pending: Mutex::new(FastMap::default()),
            read_task: Mutex::new(None),
            write_task: Mutex::new(None),
        });
        let read_inner = Arc::clone(&inner);
        let write_inner = Arc::clone(&inner);
        *inner.read_task.lock() = Some(thread::spawn(move || read_inner.read_loop(reader)));
        *inner.write_task.lock() = Some(thread::spawn(move || {
            write_inner.write_loop(writer, outbound_rx);
        }));
        Self { inner }
    }

    /// Subscribes to inbound events that are not matched by local handlers.
    pub fn subscribe(&self) -> BroadcastReceiver<InboundEvent> {
        self.inner.events.subscribe()
    }

    /// Sends a JSON-RPC request and deserializes the response.
    pub async fn request<Params, Response>(&self, method: &str, params: Params) -> Result<Response>
    where
        Params: Serialize,
        Response: DeserializeOwned,
    {
        let value = serde_json::to_value(params)?;
        let value = self.request_value(method, value).await?;
        Ok(serde_json::from_value(value)?)
    }

    /// Sends a JSON-RPC request and returns the raw JSON value.
    pub async fn request_value(&self, method: &str, params: Value) -> Result<Value> {
        if self.inner.closed.load(Ordering::SeqCst) {
            return Err(TsgoError::Closed("jsonrpc connection"));
        }
        let id = RequestId::integer(self.inner.next_id.fetch_add(1, Ordering::SeqCst) + 1);
        let (tx, rx) = mpsc::sync_channel(1);
        self.inner.pending.lock().insert(id.clone(), tx);
        self.inner
            .send_outbound(RawMessage::request(id, CompactString::from(method), params))?;
        rx.recv().map_err(|_| TsgoError::Closed("jsonrpc waiter"))?
    }

    /// Sends a JSON-RPC notification.
    pub fn notify<Params>(&self, method: &str, params: Params) -> Result<()>
    where
        Params: Serialize,
    {
        let params = serde_json::to_value(params)?;
        self.inner.send_outbound(RawMessage::notification(
            CompactString::from(method),
            params,
        ))?;
        Ok(())
    }

    /// Sends a successful response for an inbound request.
    pub fn respond<ResultBody>(&self, id: RequestId, body: ResultBody) -> Result<()>
    where
        ResultBody: Serialize,
    {
        let result = serde_json::to_value(body)?;
        self.inner.send_outbound(RawMessage::response(id, result))?;
        Ok(())
    }

    /// Sends an error response for an inbound request.
    pub fn respond_error(&self, id: RequestId, error: RpcResponseError) -> Result<()> {
        self.inner.send_outbound(RawMessage::error(id, error))?;
        Ok(())
    }

    /// Closes the connection and fails outstanding requests.
    pub async fn close(&self) -> Result<()> {
        if self.inner.closed.swap(true, Ordering::SeqCst) {
            return Ok(());
        }
        self.inner
            .fail_pending(TsgoError::Closed("jsonrpc connection"));
        self.inner.outbound.lock().take();
        if let Some(task) = self.inner.write_task.lock().take() {
            let _ = task.join();
        }
        self.inner.read_task.lock().take();
        Ok(())
    }
}

impl Inner {
    fn read_loop<R>(self: Arc<Self>, mut reader: R)
    where
        R: BufRead + Send + 'static,
    {
        loop {
            let payload = match read_frame(&mut reader) {
                Ok(payload) => payload,
                Err(err) => {
                    self.fail_pending(err);
                    return;
                }
            };
            let message: RawMessage = match serde_json::from_slice(&payload) {
                Ok(message) => message,
                Err(err) => {
                    self.fail_pending(err.into());
                    return;
                }
            };
            match message.kind() {
                Ok(MessageKind::Response { id, result, error }) => {
                    if let Some(tx) = self.pending.lock().remove(&id) {
                        let _ = tx.send(match error {
                            Some(error) => Err(TsgoError::Rpc(error)),
                            None => Ok(result.unwrap_or(Value::Null)),
                        });
                    }
                }
                Ok(MessageKind::Request { id, method, params }) => {
                    if let Some(handler) = self.handlers.get(method.as_str()) {
                        let response = (handler)(params);
                        let message = match response {
                            Ok(value) => RawMessage::response(id, value),
                            Err(error) => RawMessage::error(id, error),
                        };
                        let _ = self.send_outbound(message);
                    } else {
                        let _ = self
                            .events
                            .send(InboundEvent::Request { id, method, params });
                    }
                }
                Ok(MessageKind::Notification { method, params }) => {
                    if let Some(handler) = self.handlers.get(method.as_str()) {
                        let _ = (handler)(params);
                    } else {
                        let _ = self
                            .events
                            .send(InboundEvent::Notification { method, params });
                    }
                }
                Err(err) => {
                    self.fail_pending(err);
                    return;
                }
            }
        }
    }

    fn write_loop<W>(self: Arc<Self>, mut writer: W, outbound_rx: mpsc::Receiver<RawMessage>)
    where
        W: Write + Send + 'static,
    {
        while let Ok(message) = outbound_rx.recv() {
            let body = match serde_json::to_vec(&message) {
                Ok(body) => body,
                Err(err) => {
                    self.fail_pending(err.into());
                    return;
                }
            };
            if let Err(err) = write_frame(&mut writer, &body) {
                self.fail_pending(err);
                return;
            }
        }
    }

    fn send_outbound(&self, message: RawMessage) -> Result<()> {
        self.outbound
            .lock()
            .as_ref()
            .ok_or(TsgoError::Closed("jsonrpc writer"))?
            .send(message)
            .map_err(|_| TsgoError::Closed("jsonrpc writer"))
    }

    fn fail_pending(&self, error: TsgoError) {
        for (_, tx) in self.pending.lock().drain() {
            let _ = tx.send(Err(error.clone_for_pending()));
        }
    }
}
