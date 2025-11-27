use anyhow::Result;
use cgt_core::Transaction;
use cgt_core::calculator::calculate;
use cgt_core::parser::parse_file;
use clap::{Parser}; // Subcommand is in commands.rs
mod commands; // Declare the commands module
use commands::Commands; // Import the Commands enum
use schemars::schema_for;
use std::fs;
// use std::path::PathBuf; // Removed as it's only used in commands.rs

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
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
