use anyhow::{Result, bail};
use cgt::calculate;
use cgt::dsl::parse;
use cgt::money::FxRates;
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

/// Load the embedded configuration, then apply overrides from `./config.toml`
/// and `~/.config/cgt-tool/config.toml` in that order. An unreadable or
/// malformed auto-discovered file is skipped with a warning, not a hard error,
/// so a stray config in the working directory cannot abort the run.
fn load_config() -> Result<cgt::Config> {
    let mut config = cgt::Config::embedded()?;

    let mut paths = vec![std::path::PathBuf::from("config.toml")];
    if let Some(home) = std::env::var_os("HOME") {
        paths.push(
            std::path::PathBuf::from(home)
                .join(".config")
                .join("cgt-tool")
                .join("config.toml"),
        );
    }

    for path in paths {
        if !path.exists() {
            continue;
        }
        let outcome = match fs::read_to_string(&path) {
            Ok(text) => config
                .apply_overrides_toml(&text)
                .map_err(|e| e.to_string()),
            Err(e) => Err(e.to_string()),
        };
        if let Err(e) = outcome {
            eprintln!("WARNING: skipping config override {}: {e}", path.display());
        }
    }

    Ok(config)
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match &cli.command {
        Commands::Parse { files, schema } => {
            if *schema {
                let schema = schema_for!(Vec<cgt::Transaction>);
                println!("{}", serde_json::to_string_pretty(&schema)?);
                return Ok(());
            }

            if !files.is_empty() {
                let content = read_and_concatenate_files(files)?;
                let transactions = parse(&content)?;
                let json = serde_json::to_string_pretty(&transactions)?;
                println!("{}", json);
            }
        }
        Commands::Report {
            files,
            year,
            format,
            output,
            allow_missing_exemption,
        } => {
            let content = read_and_concatenate_files(files)?;

            let fx_rates = FxRates::bundled();

            let transactions = parse(&content)?;

            let mut config = load_config()?;
            config.allow_missing_exemption = *allow_missing_exemption;
            let report = calculate(&transactions, *year, Some(&fx_rates), &config)?;
            for warning in &report.warnings {
                eprintln!("WARNING: {warning}");
            }

            match format {
                OutputFormat::Plain => {
                    let content = cgt::format::plain::render(&report);
                    if let Some(path) = output {
                        fs::write(path, content)?;
                    } else {
                        print!("{}", content);
                    }
                }
                OutputFormat::Json => {
                    let content = serde_json::to_string_pretty(&report)?;
                    if let Some(path) = output {
                        fs::write(path, content)?;
                    } else {
                        println!("{}", content);
                    }
                }
                OutputFormat::Pdf => {
                    let pdf_bytes = cgt_pdf::render(&report)?;
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
                    transactions: transactions_path,
                    awards: awards_path,
                    output,
                } => {
                    use cgt_converter::BrokerConverter;
                    use cgt_converter::schwab::{SchwabConverter, SchwabInput};

                    let transactions_json = fs::read_to_string(transactions_path)?;
                    let awards_json = awards_path.as_ref().map(fs::read_to_string).transpose()?;

                    let converter = SchwabConverter::new();
                    let input = SchwabInput {
                        transactions_json,
                        awards_json,
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
