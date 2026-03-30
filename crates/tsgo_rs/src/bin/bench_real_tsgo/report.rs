use std::{fs, path::Path};

use serde_json::json;
use tsgo_rs::Result;

use crate::{args::Cli, dataset::DatasetCase, scenario::ScenarioRow};

pub fn write(path: &Path, cli: &Cli, datasets: &[DatasetCase], rows: &[ScenarioRow]) -> Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let datasets = datasets
        .iter()
        .map(|dataset| {
            json!({
                "label": dataset.label.as_str(),
                "configPath": dataset.config_path,
                "fileCount": dataset.file_count,
                "totalBytes": dataset.total_bytes,
                "totalLines": dataset.total_lines,
                "primaryFile": dataset.primary_file.as_str(),
            })
        })
        .collect::<Vec<_>>();
    let rows = rows
        .iter()
        .map(|row| {
            json!({
                "mode": row.mode.as_str(),
                "dataset": row.dataset.as_str(),
                "scenario": row.scenario.as_str(),
                "medianMs": row.stats.median_ms(),
                "p95Ms": row.stats.p95_ms(),
                "meanMs": row.stats.mean_ms(),
                "minMs": row.stats.min_ms(),
                "maxMs": row.stats.max_ms(),
            })
        })
        .collect::<Vec<_>>();
    fs::write(
        path,
        serde_json::to_vec_pretty(&json!({
            "tsgoPath": cli.tsgo_path,
            "coldIterations": cli.cold_iterations,
            "warmIterations": cli.warm_iterations,
            "datasets": datasets,
            "rows": rows,
        }))?,
    )?;
    Ok(())
}
