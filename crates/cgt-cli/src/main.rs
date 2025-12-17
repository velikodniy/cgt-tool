use anyhow::{Result, bail};
use cgt_core::Transaction;
use cgt_core::calculator::calculate;
use cgt_core::parser::parse_file;
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

/// Read and concatenate multiple input files.
/// Returns the combined content with newlines between files.
fn read_and_concatenate_files(files: &[std::path::PathBuf]) -> Result<String> {
    let mut contents = Vec::with_capacity(files.len());
    for path in files {
        let content = fs::read_to_string(path)?;
        contents.push(content);
    }
    Ok(contents.join("\n"))
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
        Commands::Parse { files, schema } => {
            if *schema {
                let schema = schema_for!(Vec<Transaction>);
                println!("{}", serde_json::to_string_pretty(&schema)?);
                return Ok(());
            }

            if !files.is_empty() {
                let content = read_and_concatenate_files(files)?;
                let transactions = parse_file(&content)?;
                let json = serde_json::to_string_pretty(&transactions)?;
                println!("{}", json);
            }
        }
        Commands::Report {
            files,
            year,
            format,
            output,
            fx_folder,
        } => {
            let content = read_and_concatenate_files(files)?;

            // Load FX cache (bundled by default, override if folder provided)
            let fx_cache = if let Some(folder) = fx_folder {
                let folder_files = read_fx_folder(folder)?;
                load_cache_with_overrides(folder_files)?
            } else {
                load_default_cache()?
            };

            let transactions = parse_file(&content)?;

            let report = calculate(transactions.clone(), *year, Some(&fx_cache))?;

            match format {
                OutputFormat::Plain => {
                    print!("{}", cgt_formatter_plain::format(&report, &transactions)?);
                }
                OutputFormat::Json => {
                    println!("{}", serde_json::to_string_pretty(&report)?);
                }
                OutputFormat::Pdf => {
                    let pdf_bytes = cgt_formatter_pdf::format(&report, &transactions)?;
                    let (output_path, is_default) = match output {
                        Some(p) => (p.clone(), false),
                        None => {
                            // Use first file's name for single input, "report.pdf" for multiple
                            let default_path = if files.len() == 1 {
                                files[0].with_extension("pdf")
                            } else {
                                std::path::PathBuf::from("report.pdf")
                            };
                            (default_path, true)
                        }
                    };

                    // Refuse to overwrite existing files when using default output path
                    if is_default && output_path.exists() {
                        bail!(
                            "Output file '{}' already exists. Use --output to specify a different path.",
                            output_path.display()
                        );
                    }

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
