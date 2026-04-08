mod support;

use corsa::{
    jsonrpc::InboundEvent,
    lsp::{InitializeApiSessionParams, LspClient, VirtualChange, VirtualDocument},
    runtime::block_on,
};
use serde_json::{Value, json};
use std::{future::Future, str::FromStr, thread, time::Duration};

struct OverlayStateRequest;

impl lsp_types::request::Request for OverlayStateRequest {
    type Params = Value;
    type Result = Value;
    const METHOD: &'static str = "custom/overlayState";
}

struct InitializeRequest;

impl lsp_types::request::Request for InitializeRequest {
    type Params = Value;
    type Result = Value;
    const METHOD: &'static str = "initialize";
}

struct LastConfigurationRequest;

impl lsp_types::request::Request for LastConfigurationRequest {
    type Params = Value;
    type Result = Value;
    const METHOD: &'static str = "custom/lastConfiguration";
}

struct InitializedNotification;

impl lsp_types::notification::Notification for InitializedNotification {
    type Params = Value;
    const METHOD: &'static str = "initialized";
}

fn run_async_test<F>(future: F)
where
    F: Future<Output = ()> + Send + 'static,
{
    // The LSP integration path is a little stack-heavier on Linux runners than
    // on local macOS, so use the same larger test stack we already rely on in
    // the orchestrator integration suite.
    let handle = thread::Builder::new()
        .name("lsp-stdio-test".into())
        .stack_size(8 * 1024 * 1024)
        .spawn(move || block_on(future))
        .unwrap();
    if let Err(panic) = handle.join() {
        std::panic::resume_unwind(panic);
    }
}

#[test]
fn lsp_initialize_and_custom_api_session() {
    run_async_test(async {
        let client = LspClient::spawn(support::lsp_config()).await.unwrap();
        let events = client.subscribe();
        let init = client
            .request::<InitializeRequest>(json!({}))
            .await
            .unwrap();
        assert_eq!(init["capabilities"]["textDocumentSync"], json!(1));
        let notification = events.recv_timeout(Duration::from_secs(1)).unwrap();
        match notification {
            InboundEvent::Notification { method, params } => {
                assert_eq!(method, "window/logMessage");
                assert_eq!(params["message"], json!("mock initialized"));
            }
            _ => panic!("expected notification"),
        }
        let session = client
            .initialize_api_session(InitializeApiSessionParams::default())
            .await
            .unwrap();
        assert_eq!(session.session_id, "session-1");
        client.close().await.unwrap();
    });
}

#[test]
fn lsp_server_requests_are_exposed() {
    run_async_test(async {
        let client = LspClient::spawn(support::lsp_config()).await.unwrap();
        let events = client.subscribe();
        let _: Value = client
            .request::<InitializeRequest>(json!({}))
            .await
            .unwrap();
        let _ = events.recv().unwrap();
        client.notify::<InitializedNotification>(json!({})).unwrap();
        let (id, method) = loop {
            match events.recv_timeout(Duration::from_secs(1)).unwrap() {
                InboundEvent::Request { id, method, .. } => break (id, method),
                InboundEvent::Notification { .. } => continue,
            }
        };
        assert_eq!(method, "workspace/configuration");
        client
            .respond(id, json!([{ "format": { "semicolons": "insert" } }]))
            .unwrap();
        let stored = client
            .request::<LastConfigurationRequest>(json!({}))
            .await
            .unwrap();
        assert_eq!(stored[0]["format"]["semicolons"], json!("insert"));
        client.close().await.unwrap();
    });
}

#[test]
fn lsp_overlay_tracks_virtual_documents() {
    run_async_test(async {
        let client = LspClient::spawn(support::lsp_config()).await.unwrap();
        let overlay = client.overlay();
        let events = client.subscribe();
        let _: Value = client
            .request::<InitializeRequest>(json!({}))
            .await
            .unwrap();
        let _ = events.recv().unwrap();
        let document =
            VirtualDocument::untitled("/virtual/demo.ts", "typescript", "const value = 1;\n")
                .unwrap();
        overlay.open(document.clone()).unwrap();
        let updated = overlay
            .change(
                &document.uri,
                vec![VirtualChange::splice(
                    lsp_types::Range::new(
                        lsp_types::Position::new(0, 14),
                        lsp_types::Position::new(0, 15),
                    ),
                    "2",
                )],
            )
            .unwrap();
        assert_eq!(updated.text, "const value = 2;\n");
        let state = client
            .request::<OverlayStateRequest>(json!({}))
            .await
            .unwrap();
        assert_eq!(state["documents"][0]["uri"], json!(document.uri));
        assert_eq!(state["documents"][0]["version"], json!(2));
        assert_eq!(state["documents"][0]["text"], json!("const value = 2;\n"));
        overlay.close(&document.uri).unwrap();
        let cleared = client
            .request::<OverlayStateRequest>(json!({}))
            .await
            .unwrap();
        assert_eq!(cleared["documents"], json!([]));
        client.close().await.unwrap();
    });
}

#[test]
fn lsp_overlay_rejects_duplicate_open_and_unknown_change() {
    run_async_test(async {
        let client = LspClient::spawn(support::lsp_config()).await.unwrap();
        let overlay = client.overlay();
        let _: Value = client
            .request::<InitializeRequest>(json!({}))
            .await
            .unwrap();
        let document =
            VirtualDocument::untitled("/virtual/demo.ts", "typescript", "const value = 1;\n")
                .unwrap();
        overlay.open(document.clone()).unwrap();

        let duplicate = overlay.open(document.clone()).unwrap_err();
        assert!(duplicate.to_string().contains("already open"));

        let unknown = overlay
            .change(
                &lsp_types::Uri::from_str("untitled:/virtual/missing.ts").unwrap(),
                [VirtualChange::replace("x")],
            )
            .unwrap_err();
        assert!(unknown.to_string().contains("unknown virtual document"));
        client.close().await.unwrap();
    });
}

#[test]
fn lsp_overlay_close_of_missing_document_is_a_noop() {
    run_async_test(async {
        let client = LspClient::spawn(support::lsp_config()).await.unwrap();
        let overlay = client.overlay();
        let missing = lsp_types::Uri::from_str("untitled:/virtual/missing.ts").unwrap();
        assert!(overlay.close(&missing).unwrap().is_none());
        client.close().await.unwrap();
    });
}
