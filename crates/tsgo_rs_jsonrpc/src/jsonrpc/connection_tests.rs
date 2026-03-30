use super::{InboundEvent, JsonRpcConnection, RpcHandlerMap};
use serde_json::json;
use std::{io::BufReader, os::unix::net::UnixStream, thread};

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
