use anyhow::{Result, bail};
use cgt_core::Transaction;
use cgt_core::calculator::calculate;
use cgt_core::parser::{parse_file, parse_file_with_fx};
use cgt_core::validate;
use cgt_money::{RateFile, load_cache_with_overrides, load_default_cache};
use clap::Parser;
mod commands;
use commands::{BrokerCommands, Commands, OutputFormat};
use schemars::schema_for;
use std::fs;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

fn read_fx_folder(path: &std::path::Path) -> Result<Vec<RateFile>> {
    let mut files = Vec::new();
    for entry in fs::read_dir(path)? {
        let entry = entry?;
        let file_path = entry.path();
        if file_path.extension().and_then(|e| e.to_str()) != Some("xml") {
            continue;
        }
        let xml = fs::read_to_string(&file_path)?;
        let modified = fs::metadata(&file_path).and_then(|m| m.modified()).ok();
        files.push(RateFile {
            name: file_path,
            modified,
            xml,
        });
    }
    Ok(files)
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
            fx_folder,
        } => {
            let content = fs::read_to_string(file)?;

            // Load FX cache if a folder is specified, otherwise parse without FX
            let transactions = if let Some(folder) = fx_folder {
                let folder_files = read_fx_folder(folder)?;
                let fx_cache = load_cache_with_overrides(folder_files)?;
                parse_file_with_fx(&content, Some(&fx_cache))?
            } else {
                // Try parsing without FX first
                match parse_file(&content) {
                    Ok(txns) => txns,
                    Err(cgt_core::CgtError::MissingFxRate { .. }) => {
                        // If we hit a missing FX rate, try loading bundled cache
                        let fx_cache = load_default_cache()?;
                        parse_file_with_fx(&content, Some(&fx_cache))?
                    }
                    Err(e) => return Err(e.into()),
                }
            };

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
        Commands::Convert { broker } => {
            match broker {
                BrokerCommands::Schwab {
                    transactions,
                    awards,
                    output,
                } => {
                    use cgt_converter::BrokerConverter;
                    use cgt_converter::schwab::{AwardsFormat, SchwabConverter, SchwabInput};

                    // Read transactions CSV
                    let transactions_csv = fs::read_to_string(transactions)?;

                    // Read awards file if provided and determine format
                    let (awards_content, awards_format) = if let Some(awards_path) = awards {
                        let content = fs::read_to_string(awards_path)?;
                        let format = match awards_path.extension().and_then(|e| e.to_str()) {
                            Some("json") => AwardsFormat::Json,
                            Some("csv") => AwardsFormat::Csv,
                            _ => bail!("Awards file must have .json or .csv extension"),
                        };
                        (Some(content), Some(format))
                    } else {
                        (None, None)
                    };

                    // Convert
                    let converter = SchwabConverter::new();
                    let input = SchwabInput {
                        transactions_csv,
                        awards_content,
                        awards_format,
                    };

                    let result = converter.convert(&input)?;

                    // Print warnings to stderr
                    for warning in &result.warnings {
                        eprintln!("WARNING: {}", warning);
                    }

                    // Write output
                    if let Some(output_path) = output {
                        fs::write(output_path, &result.cgt_content)?;
                    } else {
                        println!("{}", result.cgt_content);
                    }
                }
            }
        }
    }

    Ok(())
}
