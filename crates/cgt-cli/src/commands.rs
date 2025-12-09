use clap::{Subcommand, ValueEnum};
use std::path::PathBuf;

/// Output format for the tax report.
#[derive(ValueEnum, Clone, Default, Debug)]
pub enum OutputFormat {
    /// Plain text format (human-readable)
    #[default]
    Plain,
    /// JSON format (machine-readable)
    Json,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Parse a transaction file and output JSON
    Parse {
        /// Input file path
        #[arg(required_unless_present = "schema")]
        file: Option<PathBuf>,

        /// Output JSON schema
        #[arg(long)]
        schema: bool,
    },
    /// Generate tax report
    Report {
        /// Input file path
        file: PathBuf,

        /// Tax year start (e.g. 2018 for 2018/2019)
        #[arg(long)]
        year: i32,

        /// Output format (plain or json)
        #[arg(long, short, value_enum, default_value_t = OutputFormat::Plain)]
        format: OutputFormat,
    },
}
