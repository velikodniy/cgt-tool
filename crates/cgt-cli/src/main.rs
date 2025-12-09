use anyhow::{Result, bail};
use cgt_core::Transaction;
use cgt_core::calculator::calculate;
use cgt_core::parser::parse_file;
use cgt_core::validate;
use clap::Parser;
mod commands;
use commands::{Commands, OutputFormat};
use schemars::schema_for;
use std::fs;

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
        Commands::Report {
            file,
            year,
            format,
            output,
        } => {
            let content = fs::read_to_string(file)?;
            let transactions = parse_file(&content)?;

            // Validate transactions before calculation
            let validation = validate(&transactions);

            // Print warnings
            for warning in &validation.warnings {
                eprintln!("{}", warning);
            }

            // Bail on errors
            if !validation.is_valid() {
                for error in &validation.errors {
                    eprintln!("{}", error);
                }
                bail!(
                    "Validation failed with {} error(s)",
                    validation.errors.len()
                );
            }

            let report = calculate(transactions.clone(), *year)?;

            match format {
                OutputFormat::Plain => {
                    print!("{}", cgt_formatter_plain::format(&report, &transactions)?);
                }
                OutputFormat::Json => {
                    println!("{}", serde_json::to_string_pretty(&report)?);
                }
                OutputFormat::Pdf => {
                    let pdf_bytes = cgt_formatter_pdf::format(&report, &transactions)?;
                    let output_path = output.clone().unwrap_or_else(|| file.with_extension("pdf"));
                    fs::write(&output_path, pdf_bytes)?;
                    println!("PDF written to {}", output_path.display());
                }
            }
        }
    }

    Ok(())
}
