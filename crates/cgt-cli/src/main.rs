use anyhow::Result;
use cgt_core::Transaction;
use cgt_core::calculator::calculate;
use cgt_core::parser::parse_file;
use clap::{Parser, Subcommand};
use schemars::schema_for;
use std::fs;
use std::path::PathBuf;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
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
        #[arg(long, default_value = "2018")]
        year: i32,
    },
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match &cli.command {
        Commands::Parse { file, schema } => {
            if *schema {
                let schema = schema_for!(Vec<Transaction>);
                println!("{}", serde_json::to_string_pretty(&schema)?);
                return Ok(());
            }

            if let Some(path) = file {
                let content = fs::read_to_string(path)?;
                let transactions = parse_file(&content)?;
                let json = serde_json::to_string_pretty(&transactions)?;
                println!("{}", json);
            }
        }
        Commands::Report { file, year } => {
            let content = fs::read_to_string(file)?;
            let transactions = parse_file(&content)?;
            let report = calculate(transactions, *year)?;
            println!("{}", serde_json::to_string_pretty(&report)?);
        }
    }

    Ok(())
}
