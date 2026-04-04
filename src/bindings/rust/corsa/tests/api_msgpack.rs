mod support;

use corsa::api::{ApiClient, ApiMode, UpdateSnapshotParams};
use corsa::runtime::block_on;
use serde_json::json;

#[test]
fn msgpack_api_roundtrip_core() {
    block_on(async {
        let client = ApiClient::spawn(support::api_config(ApiMode::SyncMsgpackStdio))
            .await
            .unwrap();
        let init = client.initialize().await.unwrap();
        assert!(init.use_case_sensitive_file_names);
        let snapshot = client
            .update_snapshot(UpdateSnapshotParams {
                open_project: Some("/workspace/tsconfig.json".into()),
                file_changes: None,
            })
            .await
            .unwrap();
        let project = snapshot.projects[0].id.clone();
        let source = client
            .get_source_file(
                snapshot.handle.clone(),
                project.clone(),
                "/workspace/src/index.ts",
            )
            .await
            .unwrap()
            .unwrap();
        assert_eq!(source.as_bytes(), b"source-file");
        let type_node = client
            .type_to_type_node(
                snapshot.handle.clone(),
                project,
                corsa::api::TypeHandle("t0000000000000001".into()),
                None,
                None,
            )
            .await
            .unwrap()
            .unwrap();
        assert_eq!(type_node.as_bytes(), b"type-node");
        assert_eq!(
            client
                .raw_json_request("ping", serde_json::Value::Null)
                .await
                .unwrap(),
            json!("pong")
        );
        client.close().await.unwrap();
    });
}

#[test]
fn msgpack_api_callbacks_work() {
    block_on(async {
        let client = ApiClient::spawn(
            support::api_config(ApiMode::SyncMsgpackStdio)
                .with_filesystem(support::virtual_fs(&[("/virtual/tsconfig.json", "{}")])),
        )
        .await
        .unwrap();
        let config = client
            .parse_config_file("/virtual/tsconfig.json")
            .await
            .unwrap();
        assert_eq!(config.options["virtual"], json!(true));
        client.close().await.unwrap();
    });
}
