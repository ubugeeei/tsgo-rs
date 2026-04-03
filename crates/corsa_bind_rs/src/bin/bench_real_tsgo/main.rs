mod args;
mod dataset;
mod measure;
mod report;
mod scenario;
mod stats;

use std::process::ExitCode;

use corsa_bind_rs::runtime::block_on;

fn main() -> ExitCode {
    let cli = match args::parse() {
        Ok(Some(cli)) => cli,
        Ok(None) => return ExitCode::SUCCESS,
        Err(message) => {
            eprintln!("{message}");
            return ExitCode::FAILURE;
        }
    };
    match block_on(run(cli)) {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("{error}");
            ExitCode::FAILURE
        }
    }
}

async fn run(cli: args::Cli) -> corsa_bind_rs::Result<()> {
    let datasets = dataset::load(&cli).await?;
    let results = scenario::run(&cli, &datasets).await?;
    println!("tsgo: {}", cli.tsgo_path.display());
    println!("cold_iterations: {}", cli.cold_iterations);
    println!("warm_iterations: {}", cli.warm_iterations);
    println!();
    println!("dataset\tfiles\tbytes\tlines\tconfig");
    for dataset in &datasets {
        println!(
            "{}\t{}\t{}\t{}\t{}",
            dataset.label,
            dataset.file_count,
            dataset.total_bytes,
            dataset.total_lines,
            dataset.config_path.display()
        );
    }
    println!();
    println!(
        "mode\tdataset\tscenario\tsamples\tmedian_ms\tp95_ms\tp99_ms\tmean_ms\tstddev_ms\tcv_pct\tmin_ms\tmax_ms"
    );
    for row in &results {
        println!(
            "{}\t{}\t{}\t{}\t{:.3}\t{:.3}\t{:.3}\t{:.3}\t{:.3}\t{:.2}\t{:.3}\t{:.3}",
            row.mode,
            row.dataset,
            row.scenario,
            row.stats.sample_count(),
            row.stats.median_ms(),
            row.stats.p95_ms(),
            row.stats.p99_ms(),
            row.stats.mean_ms(),
            row.stats.stddev_ms(),
            row.stats.cv_percent(),
            row.stats.min_ms(),
            row.stats.max_ms()
        );
    }
    println!();
    println!("dataset\tscenario\tmsgpack_median_ms\tjsonrpc_median_ms\tspeedup_x\tp95_ratio");
    for line in report::comparison_lines(&results) {
        println!("{line}");
    }
    if let Some(path) = &cli.json_output_path {
        report::write(path, &cli, &datasets, &results)?;
    }
    Ok(())
}
