mod support;

use serde_json::{Value, json};
use std::{
    sync::{
        Arc,
        atomic::{AtomicUsize, Ordering},
    },
    time::Duration,
};
use tsgo_rs::{
    api::{ApiMode, UpdateSnapshotParams},
    lsp::{VirtualChange, VirtualDocument},
    orchestrator::{ApiOrchestrator, DistributedApiOrchestrator, RaftCluster, ReplicatedCommand},
    runtime::block_on,
};

#[test]
fn orchestrator_caches_snapshots_and_results() {
    block_on(async {
        let orchestrator = ApiOrchestrator::default();
        let profile = support::api_profile("async-cache", ApiMode::AsyncJsonRpcStdio);
        let snapshot_a = orchestrator
            .cached_snapshot(
                &profile,
                "workspace",
                UpdateSnapshotParams {
                    open_project: Some("/workspace/tsconfig.json".into()),
                    file_changes: None,
                },
            )
            .await
            .unwrap();
        let snapshot_b = orchestrator
            .cached_snapshot(
                &profile,
                "workspace",
                UpdateSnapshotParams {
                    open_project: Some("/workspace/tsconfig.json".into()),
                    file_changes: None,
                },
            )
            .await
            .unwrap();
        assert!(Arc::ptr_eq(&snapshot_a, &snapshot_b));

        let calls = Arc::new(AtomicUsize::new(0));
        let first = orchestrator
            .cached(&profile, "ping", Some(Duration::from_secs(30)), {
                let calls = calls.clone();
                move |client| async move {
                    calls.fetch_add(1, Ordering::SeqCst);
                    client.raw_json_request("ping", Value::Null).await
                }
            })
            .await
            .unwrap();
        let second = orchestrator
            .cached(&profile, "ping", Some(Duration::from_secs(30)), {
                let calls = calls.clone();
                move |client| async move {
                    calls.fetch_add(1, Ordering::SeqCst);
                    client.raw_json_request("ping", Value::Null).await
                }
            })
            .await
            .unwrap();
        assert_eq!(first, json!("pong"));
        assert_eq!(second, json!("pong"));
        assert_eq!(calls.load(Ordering::SeqCst), 1);
    });
}

#[test]
fn orchestrator_executes_parallel_batches() {
    block_on(async {
        let orchestrator = ApiOrchestrator::default();
        let profile = support::api_profile("async-batch", ApiMode::AsyncJsonRpcStdio);
        let values = orchestrator
            .execute_all(&profile, 2, [1_u32, 2, 3, 4], |client, value| async move {
                let echoed = client
                    .raw_json_request("echo", json!({ "value": value }))
                    .await?;
                Ok::<_, tsgo_rs::TsgoError>(echoed["value"].as_u64().unwrap() as u32)
            })
            .await
            .unwrap();
        assert_eq!(values.as_slice(), &[1, 2, 3, 4]);
    });
}

#[test]
fn raft_cluster_elects_a_leader_and_rejects_follower_writes() {
    let cluster = RaftCluster::new(["n1", "n2", "n3"]);
    let document =
        VirtualDocument::in_memory("cluster", "/main.ts", "typescript", "let value = 1;").unwrap();
    assert!(
        cluster
            .append(
                "n1",
                ReplicatedCommand::PutDocument {
                    document: document.clone(),
                },
            )
            .is_err()
    );
    assert_eq!(cluster.campaign("n2").unwrap(), 1);
    cluster
        .append(
            "n2",
            ReplicatedCommand::PutDocument {
                document: document.clone(),
            },
        )
        .unwrap();
    assert!(
        cluster
            .append(
                "n1",
                ReplicatedCommand::PutDocument {
                    document: document.clone(),
                },
            )
            .is_err()
    );
    for node in ["n1", "n2", "n3"] {
        let state = cluster.node_state(node).unwrap();
        assert_eq!(state.documents[document.uri.as_str()], document);
    }
}

#[test]
fn distributed_orchestrator_replicates_virtual_documents_and_results() {
    block_on(async {
        let orchestrator = DistributedApiOrchestrator::new(["n1", "n2", "n3"]);
        let profile = support::api_profile("dist-cache", ApiMode::AsyncJsonRpcStdio);
        let document =
            VirtualDocument::in_memory("cluster", "/main.ts", "typescript", "let value = 1;")
                .unwrap();
        orchestrator.campaign("n1").unwrap();
        orchestrator
            .open_virtual_document("n1", document.clone())
            .unwrap();
        let updated = orchestrator
            .change_virtual_document(
                "n1",
                &document.uri,
                [VirtualChange::splice(
                    lsp_types::Range::new(
                        lsp_types::Position::new(0, 12),
                        lsp_types::Position::new(0, 13),
                    ),
                    "2",
                )],
            )
            .unwrap();
        assert_eq!(updated.text, "let value = 2;");
        let calls = Arc::new(AtomicUsize::new(0));
        let first: Value = orchestrator
            .cached(&profile, "n1", "ping", Some(Duration::from_secs(30)), {
                let calls = calls.clone();
                move |client| async move {
                    calls.fetch_add(1, Ordering::SeqCst);
                    client.raw_json_request("ping", Value::Null).await
                }
            })
            .await
            .unwrap();
        let second: Value = orchestrator
            .cached(&profile, "n1", "ping", Some(Duration::from_secs(30)), {
                let calls = calls.clone();
                move |client| async move {
                    calls.fetch_add(1, Ordering::SeqCst);
                    client.raw_json_request("ping", Value::Null).await
                }
            })
            .await
            .unwrap();
        assert_eq!(first, json!("pong"));
        assert_eq!(second, json!("pong"));
        assert_eq!(calls.load(Ordering::SeqCst), 1);
        for node in ["n1", "n2", "n3"] {
            let state = orchestrator.node_state(node).unwrap();
            assert_eq!(
                state.documents[document.uri.as_str()].text,
                "let value = 2;"
            );
            assert_eq!(
                state.result::<Value>("ping").unwrap().unwrap(),
                json!("pong")
            );
        }
    });
}

#[test]
fn distributed_orchestrator_replicates_snapshot_records() {
    block_on(async {
        let orchestrator = DistributedApiOrchestrator::new(["leader", "follower-a", "follower-b"]);
        let profile = support::api_profile("dist-snapshot", ApiMode::AsyncJsonRpcStdio);
        orchestrator.campaign("leader").unwrap();
        let snapshot = orchestrator
            .cached_snapshot(
                &profile,
                "leader",
                "workspace",
                UpdateSnapshotParams {
                    open_project: Some("/workspace/tsconfig.json".into()),
                    file_changes: None,
                },
            )
            .await
            .unwrap();
        let record = orchestrator.snapshot_record("leader", "workspace").unwrap();
        assert_eq!(record.handle, snapshot.handle);
        for node in ["leader", "follower-a", "follower-b"] {
            let state = orchestrator.node_state(node).unwrap();
            assert_eq!(state.snapshots["workspace"].handle, snapshot.handle);
        }
    });
}
