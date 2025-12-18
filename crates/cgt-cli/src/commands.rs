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
    /// Start MCP (Model Context Protocol) server for AI assistant integration
    Mcp,
    /// Parse transaction file(s) and output JSON
    Parse {
        /// Input file path(s)
        #[arg(required_unless_present = "schema")]
        files: Vec<PathBuf>,

        /// Output JSON schema
        #[arg(long)]
        schema: bool,
    },
    /// Generate tax report
    Report {
        /// Input file path(s)
        #[arg(required = true)]
        files: Vec<PathBuf>,

        /// Tax year start (e.g. 2024 for 2024/25). If omitted, report includes all years.
        #[arg(long)]
        year: Option<i32>,

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
    /// Convert broker export files to CGT DSL format
    Convert {
        #[command(subcommand)]
        broker: BrokerCommands,
    },
}

#[derive(Subcommand)]
pub enum BrokerCommands {
    /// Convert Charles Schwab export files
    Schwab {
        /// Path to Schwab transactions CSV file
        transactions: PathBuf,

        /// Optional path to Schwab equity awards JSON file (required for RSU vesting)
        #[arg(long)]
        awards: Option<PathBuf>,

        /// Output file path (if not provided, prints to stdout)
        #[arg(long, short)]
        output: Option<PathBuf>,
    },
}
