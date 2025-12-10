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
    /// PDF format (printable document)
    Pdf,
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

        /// Output format (plain, json, or pdf)
        #[arg(long, short, value_enum, default_value_t = OutputFormat::Plain)]
        format: OutputFormat,

        /// Output file path (required for PDF format)
        #[arg(long, short)]
        output: Option<PathBuf>,

        /// Folder containing monthly FX rate XML files from trade-tariff.service.gov.uk.
        /// If not provided, bundled rates are used. If a required month/currency is
        /// missing from provided files, falls back to bundled rates with a warning.
        #[arg(long, value_name = "PATH")]
        fx_folder: Option<PathBuf>,
    },
}
