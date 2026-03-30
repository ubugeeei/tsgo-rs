use tsgo_rs::{
    Result,
    api::{ApiClient, ApiMode, ApiSpawnConfig, UpdateSnapshotParams},
    fast::{CompactString, SmallVec},
};

use crate::{
    args::Cli,
    dataset::DatasetCase,
    measure::{measure, measure_warm},
    stats::Stats,
};

#[derive(Clone, Debug)]
pub struct ScenarioRow {
    pub mode: CompactString,
    pub dataset: CompactString,
    pub scenario: CompactString,
    pub stats: Stats,
}

struct ProjectSession {
    client: ApiClient,
    snapshot: tsgo_rs::api::ManagedSnapshot,
    project: tsgo_rs::api::ProjectHandle,
    file: CompactString,
}

pub async fn run(cli: &Cli, datasets: &[DatasetCase]) -> Result<SmallVec<[ScenarioRow; 64]>> {
    let mut rows = SmallVec::<[ScenarioRow; 64]>::new();
    for mode in &cli.modes {
        for dataset in datasets {
            rows.extend(run_dataset(cli, dataset, *mode).await?);
        }
    }
    Ok(rows)
}

async fn run_dataset(
    cli: &Cli,
    dataset: &DatasetCase,
    mode: ApiMode,
) -> Result<SmallVec<[ScenarioRow; 10]>> {
    let mut rows = SmallVec::<[ScenarioRow; 10]>::new();
    rows.push(row(
        mode,
        dataset,
        "spawn_initialize",
        spawn_initialize(cli, mode).await?,
    ));
    rows.push(row(
        mode,
        dataset,
        "parse_config",
        parse_config(cli, dataset, mode).await?,
    ));
    rows.push(row(
        mode,
        dataset,
        "update_snapshot_cold",
        update_snapshot_cold(cli, dataset, mode).await?,
    ));
    rows.push(row(
        mode,
        dataset,
        "update_snapshot_warm",
        update_snapshot_warm(cli, dataset, mode).await?,
    ));
    let session = open_project_session(cli, dataset, mode).await?;
    rows.push(row(
        mode,
        dataset,
        "default_project",
        default_project(&session, cli.warm_iterations).await?,
    ));
    rows.push(row(
        mode,
        dataset,
        "get_source_file",
        get_source_file(&session, cli.warm_iterations).await?,
    ));
    rows.push(row(
        mode,
        dataset,
        "get_string_type",
        get_string_type(&session, cli.warm_iterations).await?,
    ));
    rows.push(row(
        mode,
        dataset,
        "type_to_string",
        type_to_string(&session, cli.warm_iterations).await?,
    ));
    session.snapshot.release().await?;
    session.client.close().await?;
    Ok(rows)
}

fn row(mode: ApiMode, dataset: &DatasetCase, scenario: &str, stats: Stats) -> ScenarioRow {
    ScenarioRow {
        mode: mode_name(mode),
        dataset: dataset.label.clone(),
        scenario: CompactString::from(scenario),
        stats,
    }
}

fn mode_name(mode: ApiMode) -> CompactString {
    match mode {
        ApiMode::AsyncJsonRpcStdio => CompactString::from("jsonrpc"),
        ApiMode::SyncMsgpackStdio => CompactString::from("msgpack"),
    }
}

fn spawn_config(cli: &Cli, mode: ApiMode) -> ApiSpawnConfig {
    ApiSpawnConfig::new(&cli.tsgo_path)
        .with_cwd(&cli.root_dir)
        .with_mode(mode)
}

async fn spawn_initialize(cli: &Cli, mode: ApiMode) -> Result<Stats> {
    measure(cli.cold_iterations, || async {
        let client = ApiClient::spawn(spawn_config(cli, mode)).await?;
        let _ = client.initialize().await?;
        client.close().await
    })
    .await
}

async fn parse_config(cli: &Cli, dataset: &DatasetCase, mode: ApiMode) -> Result<Stats> {
    let client = ApiClient::spawn(spawn_config(cli, mode)).await?;
    let _ = client.initialize().await?;
    let stats = measure_warm(cli.warm_iterations, || async {
        let _ = client
            .parse_config_file(dataset.config_wire.as_str())
            .await?;
        Ok(())
    })
    .await?;
    client.close().await?;
    Ok(stats)
}

async fn update_snapshot_cold(cli: &Cli, dataset: &DatasetCase, mode: ApiMode) -> Result<Stats> {
    measure(cli.cold_iterations, || async {
        let client = ApiClient::spawn(spawn_config(cli, mode)).await?;
        let snapshot = client
            .update_snapshot(UpdateSnapshotParams {
                open_project: Some(dataset.config_wire.to_string()),
                file_changes: None,
            })
            .await?;
        snapshot.release().await?;
        client.close().await
    })
    .await
}

async fn update_snapshot_warm(cli: &Cli, dataset: &DatasetCase, mode: ApiMode) -> Result<Stats> {
    let client = ApiClient::spawn(spawn_config(cli, mode)).await?;
    let _ = client.initialize().await?;
    let stats = measure_warm(cli.warm_iterations, || async {
        let snapshot = client
            .update_snapshot(UpdateSnapshotParams {
                open_project: Some(dataset.config_wire.to_string()),
                file_changes: None,
            })
            .await?;
        snapshot.release().await
    })
    .await?;
    client.close().await?;
    Ok(stats)
}

async fn open_project_session(
    cli: &Cli,
    dataset: &DatasetCase,
    mode: ApiMode,
) -> Result<ProjectSession> {
    let client = ApiClient::spawn(spawn_config(cli, mode)).await?;
    let snapshot = client
        .update_snapshot(UpdateSnapshotParams {
            open_project: Some(dataset.config_wire.to_string()),
            file_changes: None,
        })
        .await?;
    let project = snapshot.projects[0].id.clone();
    Ok(ProjectSession {
        client,
        snapshot,
        project,
        file: dataset.primary_file.clone(),
    })
}

async fn default_project(session: &ProjectSession, iterations: usize) -> Result<Stats> {
    measure_warm(iterations, || async {
        let _ = session
            .client
            .get_default_project_for_file(session.snapshot.handle.clone(), session.file.as_str())
            .await?;
        Ok(())
    })
    .await
}

async fn get_source_file(session: &ProjectSession, iterations: usize) -> Result<Stats> {
    measure_warm(iterations, || async {
        let _ = session
            .client
            .get_source_file(
                session.snapshot.handle.clone(),
                session.project.clone(),
                session.file.as_str(),
            )
            .await?;
        Ok(())
    })
    .await
}

async fn get_string_type(session: &ProjectSession, iterations: usize) -> Result<Stats> {
    measure_warm(iterations, || async {
        let _ = session
            .client
            .get_string_type(session.snapshot.handle.clone(), session.project.clone())
            .await?;
        Ok(())
    })
    .await
}

async fn type_to_string(session: &ProjectSession, iterations: usize) -> Result<Stats> {
    let ty = session
        .client
        .get_string_type(session.snapshot.handle.clone(), session.project.clone())
        .await?;
    measure_warm(iterations, || async {
        let _ = session
            .client
            .type_to_string(
                session.snapshot.handle.clone(),
                session.project.clone(),
                ty.id.clone(),
                None,
                None,
            )
            .await?;
        Ok(())
    })
    .await
}
