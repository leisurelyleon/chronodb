//! Command-line argument definitions.

use std::path::PathBuf;

use clap::{Parser, Subcommand, ValueEnum};

/// An embedded time-series storage engine.
#[derive(Debug, Parser)]
#[command(name = "chronodb", version, about, long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Command,
}

#[derive(Debug, Subcommand)]
pub enum Command {
    /// Append a single data point to a write-ahead log file.
    Ingest {
        #[arg(long)]
        wal: PathBuf,
        #[arg(long)]
        series: String,
        #[arg(long)]
        timestamp: u64,
        #[arg(long)]
        value: f64,
    },
    /// Replay a WAL file and run a windowed range query.
    Query {
        #[arg(long)]
        wal: PathBuf,
        #[arg(long)]
        series: String,
        #[arg(long)]
        from: u64,
        #[arg(long)]
        to: u64,
        #[arg(long)]
        window: u64,
        #[arg(long, value_enum, default_value = "avg")]
        agg: AggArg,
        #[arg(long, default_value_t = 3600)]
        partition_size: u64,
    },
    /// Run a self-contained in-memory demonstration (no files needed).
    Demo,
}

/// The aggregation to apply, as a CLI argument.
#[derive(Clone, Debug, ValueEnum)]
pub enum AggArg {
    Min,
    Max,
    Sum,
    Avg,
    Count,
}

impl AggArg {
    /// Maps the CLI argument to the core aggregation.
    pub fn to_core(&self) -> chronodb_core::Aggregation {
        match self {
            AggArg::Min => chronodb_core::Aggregation::Min,
            AggArg::Max => chronodb_core::Aggregation::Max,
            AggArg::Sum => chronodb_core::Aggregation::Sum,
            AggArg::Avg => chronodb_core::Aggregation::Avg,
            AggArg::Count => chronodb_core::Aggregation::Count,
        }
    }
}
