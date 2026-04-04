mod support;

use corsa::api::{ApiClient, ApiMode, PrintNodeOptions, UpdateSnapshotParams};
use corsa::runtime::block_on;
use serde_json::json;

#[test]
fn async_api_roundtrip_core() {
    block_on(async {
        let client = ApiClient::spawn(
            support::api_config(ApiMode::AsyncJsonRpcStdio)
                .with_allow_unstable_upstream_calls(true),
        )
        .await
        .unwrap();
        let init = client.initialize().await.unwrap();
        assert_eq!(
            init.current_directory,
            support::test_cwd().display().to_string()
        );
        let config = client
            .parse_config_file("/workspace/tsconfig.json")
            .await
            .unwrap();
        assert_eq!(config.file_names, vec!["/workspace/src/index.ts"]);
        let snapshot = client
            .update_snapshot(UpdateSnapshotParams {
                open_project: Some("/workspace/tsconfig.json".into()),
                file_changes: None,
            })
            .await
            .unwrap();
        assert_eq!(snapshot.projects.len(), 1);
        let project = snapshot.projects[0].id.clone();
        let default = snapshot
            .get_default_project_for_file("/workspace/src/index.ts")
            .await
            .unwrap()
            .unwrap();
        assert_eq!(default.id.as_str(), project.as_str());
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
        let symbol = client
            .get_symbol_at_position(
                snapshot.handle.clone(),
                project.clone(),
                "/workspace/src/index.ts",
                1,
            )
            .await
            .unwrap()
            .unwrap();
        assert_eq!(symbol.name, "value");
        let ty = client
            .get_type_of_symbol(snapshot.handle.clone(), project.clone(), symbol.id.clone())
            .await
            .unwrap()
            .unwrap();
        assert_eq!(ty.id.as_str(), "t0000000000000001");
        let rendered = client
            .type_to_string(
                snapshot.handle.clone(),
                project.clone(),
                ty.id.clone(),
                None,
                None,
            )
            .await
            .unwrap();
        assert_eq!(rendered, "type:string");
        let printed = client
            .print_node(
                &corsa::api::EncodedPayload::new(b"hello".to_vec()),
                PrintNodeOptions::default(),
            )
            .await
            .unwrap();
        assert_eq!(printed, "print:hello");
        snapshot.release().await.unwrap();
        client.close().await.unwrap();
    });
}

#[test]
fn async_api_rejects_unstable_print_node_by_default() {
    block_on(async {
        let client = ApiClient::spawn(support::api_config(ApiMode::AsyncJsonRpcStdio))
            .await
            .unwrap();
        let error = client
            .print_node(
                &corsa::api::EncodedPayload::new(b"hello".to_vec()),
                PrintNodeOptions::default(),
            )
            .await
            .unwrap_err();
        assert!(matches!(
            error,
            corsa::TsgoError::Unsupported(message) if message.contains("printNode is disabled by default")
        ));
        client.close().await.unwrap();
    });
}

#[test]
fn async_api_callbacks_work() {
    block_on(async {
        let client = ApiClient::spawn(
            support::api_config(ApiMode::AsyncJsonRpcStdio)
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

#[test]
fn async_api_full_surface_methods() {
    block_on(async {
        let client = ApiClient::spawn(support::api_config(ApiMode::AsyncJsonRpcStdio))
            .await
            .unwrap();
        let snapshot = client
            .update_snapshot(UpdateSnapshotParams {
                open_project: Some("/workspace/tsconfig.json".into()),
                file_changes: None,
            })
            .await
            .unwrap();
        let project = snapshot.projects[0].id.clone();
        let node = corsa::api::NodeHandle("1.3.80./workspace/src/index.ts".into());
        let symbol = client
            .get_symbol_at_location(snapshot.handle.clone(), project.clone(), node.clone())
            .await
            .unwrap()
            .unwrap();
        assert_eq!(
            client
                .get_symbols_at_locations(
                    snapshot.handle.clone(),
                    project.clone(),
                    vec![node.clone()]
                )
                .await
                .unwrap()[0]
                .as_ref()
                .unwrap()
                .id
                .as_str(),
            symbol.id.as_str()
        );
        assert_eq!(
            client
                .get_symbols_at_positions(
                    snapshot.handle.clone(),
                    project.clone(),
                    "/workspace/src/index.ts",
                    vec![1, 2]
                )
                .await
                .unwrap()
                .len(),
            2
        );
        assert!(
            client
                .get_declared_type_of_symbol(
                    snapshot.handle.clone(),
                    project.clone(),
                    symbol.id.clone()
                )
                .await
                .unwrap()
                .is_some()
        );
        assert!(
            client
                .resolve_name(
                    snapshot.handle.clone(),
                    project.clone(),
                    "value",
                    2,
                    Some(node.clone()),
                    None,
                    None,
                    None
                )
                .await
                .unwrap()
                .is_some()
        );
        assert!(
            client
                .get_parent_of_symbol(snapshot.handle.clone(), symbol.id.clone())
                .await
                .unwrap()
                .is_some()
        );
        assert_eq!(
            client
                .get_members_of_symbol(snapshot.handle.clone(), symbol.id.clone())
                .await
                .unwrap()
                .len(),
            1
        );
        assert_eq!(
            client
                .get_exports_of_symbol(snapshot.handle.clone(), symbol.id.clone())
                .await
                .unwrap()
                .len(),
            1
        );
        let exported = client
            .get_export_symbol_of_symbol(snapshot.handle.clone(), symbol.id.clone())
            .await
            .unwrap();
        assert_eq!(exported.name, "exported");
        let ty = client
            .get_type_at_location(snapshot.handle.clone(), project.clone(), node.clone())
            .await
            .unwrap()
            .unwrap();
        assert!(
            client
                .get_type_at_locations(snapshot.handle.clone(), project.clone(), vec![node.clone()])
                .await
                .unwrap()[0]
                .is_some()
        );
        assert!(
            client
                .get_type_at_position(
                    snapshot.handle.clone(),
                    project.clone(),
                    "/workspace/src/index.ts",
                    1
                )
                .await
                .unwrap()
                .is_some()
        );
        assert_eq!(
            client
                .get_types_at_positions(
                    snapshot.handle.clone(),
                    project.clone(),
                    "/workspace/src/index.ts",
                    vec![1, 2]
                )
                .await
                .unwrap()
                .len(),
            2
        );
        assert_eq!(
            client
                .get_signatures_of_type(snapshot.handle.clone(), project.clone(), ty.id.clone(), 0)
                .await
                .unwrap()
                .len(),
            1
        );
        assert!(
            client
                .get_contextual_type(snapshot.handle.clone(), project.clone(), node.clone())
                .await
                .unwrap()
                .is_some()
        );
        assert!(
            client
                .get_base_type_of_literal_type(
                    snapshot.handle.clone(),
                    project.clone(),
                    ty.id.clone()
                )
                .await
                .unwrap()
                .is_some()
        );
        assert!(
            client
                .get_shorthand_assignment_value_symbol(
                    snapshot.handle.clone(),
                    project.clone(),
                    node.clone()
                )
                .await
                .unwrap()
                .is_some()
        );
        assert!(
            client
                .get_type_of_symbol_at_location(
                    snapshot.handle.clone(),
                    project.clone(),
                    symbol.id.clone(),
                    node.clone()
                )
                .await
                .unwrap()
                .is_some()
        );
        assert_eq!(
            client
                .type_to_type_node(
                    snapshot.handle.clone(),
                    project.clone(),
                    ty.id.clone(),
                    Some(node.clone()),
                    None
                )
                .await
                .unwrap()
                .unwrap()
                .as_bytes(),
            b"type-node"
        );
        assert!(
            client
                .is_context_sensitive(snapshot.handle.clone(), project.clone(), node.clone())
                .await
                .unwrap()
        );
        assert!(
            client
                .get_any_type(snapshot.handle.clone(), project.clone())
                .await
                .is_ok()
        );
        assert!(
            client
                .get_string_type(snapshot.handle.clone(), project.clone())
                .await
                .is_ok()
        );
        assert!(
            client
                .get_number_type(snapshot.handle.clone(), project.clone())
                .await
                .is_ok()
        );
        assert!(
            client
                .get_boolean_type(snapshot.handle.clone(), project.clone())
                .await
                .is_ok()
        );
        assert!(
            client
                .get_void_type(snapshot.handle.clone(), project.clone())
                .await
                .is_ok()
        );
        assert!(
            client
                .get_undefined_type(snapshot.handle.clone(), project.clone())
                .await
                .is_ok()
        );
        assert!(
            client
                .get_null_type(snapshot.handle.clone(), project.clone())
                .await
                .is_ok()
        );
        assert!(
            client
                .get_never_type(snapshot.handle.clone(), project.clone())
                .await
                .is_ok()
        );
        assert!(
            client
                .get_unknown_type(snapshot.handle.clone(), project.clone())
                .await
                .is_ok()
        );
        assert!(
            client
                .get_big_int_type(snapshot.handle.clone(), project.clone())
                .await
                .is_ok()
        );
        assert!(
            client
                .get_es_symbol_type(snapshot.handle.clone(), project.clone())
                .await
                .is_ok()
        );
        client.close().await.unwrap();
    });
}
