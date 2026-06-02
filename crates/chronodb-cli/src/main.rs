//! `chronodb` command-line entry point.

use std::path::Path;
use std::process::ExitCode;

use clap::Parser;

use chronodb_cli::cli::{AggArg, Cli, Command};
use chronodb_core::{Aggregation, DataPoint, QuerySpec, Store, WindowResult};
use chronodb_store::{FileWal, WalRecord, WriteAheadLog};

fn main() -> ExitCode {
    let cli = Cli::parse();
    let result = match cli.command {
        Command::Ingest {
            wal,
            series,
            timestamp,
            value,
        } => cmd_ingest(&wal, series, timestamp, value),
        Command::Query {
            wal,
            series,
            from,
            to,
            window,
            agg,
            partition_size,
        } => cmd_query(&wal, &series, from, to, window, agg, partition_size),
        Command::Demo => {
            run_demo();
            Ok(())
        }
    };

    match result {
        Ok(()) => ExitCode::SUCCESS,
        Err(message) => {
            eprintln!("chronodb: {message}");
            ExitCode::FAILURE
        }
    }
}

fn cmd_ingest(wal_path: &Path, series: String, timestamp: u64, value: f64) -> Result<(), String> {
    let mut wal = FileWal::new(wal_path);
    wal.append(&WalRecord {
        series,
        timestamp,
        value,
    })
    .map_err(|e| e.to_string())?;
    println!("Appended point to {}.", wal_path.display());
    Ok(())
}

#[allow(clippy::too_many_arguments)]
fn cmd_query(
    wal_path: &Path,
    series: &str,
    from: u64,
    to: u64,
    window: u64,
    agg: AggArg,
    partition_size: u64,
) -> Result<(), String> {
    // Rebuild the store by replaying the WAL.
    let wal = FileWal::new(wal_path);
    let records = wal.replay().map_err(|e| e.to_string())?;

    let mut store = Store::new(partition_size);
    for record in &records {
        store.ingest(
            record.series.clone(),
            DataPoint::new(record.timestamp, record.value),
        );
    }

    let spec = QuerySpec::new(from, to, window, agg.to_core());
    let windows = store.query(series, &spec).map_err(|e| e.to_string())?;
    print_windows(series, &windows);
    Ok(())
}

fn print_windows(series: &str, windows: &[WindowResult]) {
    println!("series '{series}': {} window(s)", windows.len());
    for window in windows {
        println!(
            "  [{:>6}] value={:<10} count={}",
            window.start, window.value, window.count
        );
    }
}

/// A self-contained, deterministic demonstration: ingest a metric, downsample
/// it, then apply retention and re-query to show compaction.
fn run_demo() {
    let mut store = Store::new(60);

    // Ingest "cpu" with value = timestamp + 10, every 10 ticks over [0, 120).
    for t in (0u64..120).step_by(10) {
        store.ingest("cpu", DataPoint::new(t, (t + 10) as f64));
    }

    println!("== Ingested ==");
    println!("series 'cpu': {} points", store.point_count("cpu"));

    println!("\n== Downsample: 30-tick windows, average ==");
    let spec = QuerySpec::new(0, 120, 30, Aggregation::Avg);
    let windows = store.query("cpu", &spec).unwrap();
    print_windows("cpu", &windows);

    println!("\n== Retention: now=120, horizon=60 (drop data older than tick 60) ==");
    let dropped = store.retain(120, 60);
    println!(
        "dropped {dropped} expired partition(s); 'cpu' now holds {} points",
        store.point_count("cpu")
    );

    println!("\n== Re-query after retention ==");
    let after = store.query("cpu", &spec).unwrap();
    print_windows("cpu", &after);
}
