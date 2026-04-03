mod support;

use std::{sync::Arc, time::Duration};

use corsa_bind_rs::{
    api::{ApiMode, ApiProfile, UpdateSnapshotParams},
    orchestrator::ApiOrchestrator,
    runtime::block_on,
};
use serde_json::{Value, json};

fn main() -> Result<(), corsa_bind_rs::TsgoError> {
    let result = block_on(async {
        let orchestrator = ApiOrchestrator::default();
        let profile = ApiProfile::new(
            "orchestrator-demo",
            support::mock_api_config("orchestrator_cache", ApiMode::AsyncJsonRpcStdio)?,
        );
        orchestrator.prewarm(&profile, 2).await?;

        let snapshot_a = orchestrator
            .cached_snapshot(
                &profile,
                "workspace",
                UpdateSnapshotParams {
                    open_project: Some("/workspace/tsconfig.json".into()),
                    file_changes: None,
                },
            )
            .await?;
        let snapshot_b = orchestrator
            .cached_snapshot(
                &profile,
                "workspace",
                UpdateSnapshotParams {
                    open_project: Some("/workspace/tsconfig.json".into()),
                    file_changes: None,
                },
            )
            .await?;
        let ping: Value = orchestrator
            .cached(
                &profile,
                "ping",
                Some(Duration::from_secs(30)),
                |client| async move { client.raw_json_request("ping", Value::Null).await },
            )
            .await?;
        let echoed_values = orchestrator
            .execute_all(&profile, 2, [1_u32, 2, 3, 4], |client, value| async move {
                let echoed = client
                    .raw_json_request("echo", json!({ "value": value }))
                    .await?;
                Ok::<_, corsa_bind_rs::TsgoError>(echoed["value"].as_u64().unwrap() as u32)
            })
            .await?;
        let stats = orchestrator.stats();

        Ok::<_, corsa_bind_rs::TsgoError>(json!({
            "snapshotCacheHit": Arc::ptr_eq(&snapshot_a, &snapshot_b),
            "snapshotHandle": snapshot_a.handle,
            "cachedPing": ping,
            "echoedValues": echoed_values,
            "stats": {
                "profileCount": stats.profile_count,
                "workerCount": stats.worker_count,
                "cachedSnapshotCount": stats.cached_snapshot_count,
                "cachedResultCount": stats.cached_result_count,
            },
        }))
    })?;

    support::print_json(result);
    Ok(())
}
