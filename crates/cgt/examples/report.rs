//! Equivalence shim: runs the new engine as a binary the harness can drive.
//! Usage: report <file> --format json   |   parse <file>
//!
//! Mirrors the oracle CLI's argv and exit conventions: success prints pretty
//! JSON to stdout and exits 0; any error prints to stderr and exits nonzero.

use std::process::ExitCode;

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().collect();
    match run(&args) {
        Ok(out) => {
            println!("{out}");
            ExitCode::SUCCESS
        }
        Err(msg) => {
            eprintln!("{msg}");
            ExitCode::FAILURE
        }
    }
}

fn run(args: &[String]) -> Result<String, String> {
    let cmd = args.get(1).map(String::as_str).unwrap_or("");
    let file = args
        .get(2)
        .ok_or("usage: report|parse <file> [--format json]")?;
    let content = std::fs::read_to_string(file).map_err(|e| e.to_string())?;
    let transactions = cgt::dsl::parse(&content).map_err(|e| e.to_string())?;
    match cmd {
        "parse" => serde_json::to_string_pretty(&transactions).map_err(|e| e.to_string()),
        "report" => {
            let fx = cgt::money::load_default_cache().map_err(|e| e.to_string())?;
            let config = cgt::Config::embedded().map_err(|e| e.to_string())?;
            let report = cgt::calculate(&transactions, None, Some(&fx), &config)
                .map_err(|e| e.to_string())?;
            serde_json::to_string_pretty(&report).map_err(|e| e.to_string())
        }
        other => Err(format!("unknown command: {other}")),
    }
}
