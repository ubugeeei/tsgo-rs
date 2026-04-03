#![cfg(unix)]

use super::{InboundEvent, JsonRpcConnection, JsonRpcConnectionOptions, RpcHandlerMap};
use serde_json::json;
use std::sync::{Arc, Mutex};
use std::time::Duration;
use std::{io::BufReader, os::unix::net::UnixStream, thread};
use tsgo_rs_core::{TsgoEvent, TsgoObserver};

#[derive(Default)]
struct EventCollector {
    events: Mutex<Vec<TsgoEvent>>,
}

impl TsgoObserver for EventCollector {
    fn on_event(&self, event: &TsgoEvent) {
        self.events.lock().unwrap().push(event.clone());
    }
}

#[test]
fn routes_request_and_response() {
    let (client_socket, server_socket) = UnixStream::pair().unwrap();
    let client = JsonRpcConnection::spawn(
        BufReader::new(client_socket.try_clone().unwrap()),
        client_socket,
        RpcHandlerMap::default(),
    );
    let server = JsonRpcConnection::spawn(
        BufReader::new(server_socket.try_clone().unwrap()),
        server_socket,
        RpcHandlerMap::default(),
    );
    let events = server.subscribe();
    let waiter = thread::spawn(move || match events.recv().unwrap() {
        InboundEvent::Request { id, method, params } => {
            assert_eq!(method.as_str(), "ping");
            assert_eq!(params, json!({"value": 1}));
            server.respond(id, json!({"pong": true})).unwrap();
        }
        _ => panic!("unexpected event"),
    });
    let response: serde_json::Value =
        tsgo_rs_runtime::block_on(client.request("ping", json!({"value": 1}))).unwrap();
    waiter.join().unwrap();
    assert_eq!(response, json!({"pong": true}));
}

#[test]
fn request_times_out_when_no_response_arrives() {
    let (client_socket, _server_socket) = UnixStream::pair().unwrap();
    let observer = Arc::new(EventCollector::default());
    let client = JsonRpcConnection::spawn_with_options(
        BufReader::new(client_socket.try_clone().unwrap()),
        client_socket,
        RpcHandlerMap::default(),
        JsonRpcConnectionOptions::new()
            .with_request_timeout(Some(Duration::from_millis(10)))
            .with_observer(observer.clone()),
    );
    let error =
        tsgo_rs_runtime::block_on(client.request_value("ping", json!({"value": 1}))).unwrap_err();
    assert!(matches!(
        error,
        crate::TsgoError::Timeout(message) if message.contains("jsonrpc request `ping`")
    ));
    assert_eq!(
        observer.events.lock().unwrap().as_slice(),
        &[TsgoEvent::JsonRpcRequestTimedOut {
            method: "ping".into(),
            timeout: Duration::from_millis(10),
        }]
    );
}
