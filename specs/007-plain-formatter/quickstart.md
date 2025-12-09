# Quickstart: Plain Text Report Formatter

**Feature**: 007-plain-formatter
**Date**: 2025-12-08

## Overview

This feature adds a plain text output format to the CGT calculator CLI. The formatter is implemented as a separate crate for extensibility.

## Key Files

### New Crate: cgt-formatter-plain

```text
crates/cgt-formatter-plain/
├── Cargo.toml
└── src/
    └── lib.rs
```

### Modified Files

- `Cargo.toml` (workspace) - Add new crate member
- `crates/cgt-cli/Cargo.toml` - Add formatter dependency
- `crates/cgt-cli/src/commands.rs` - Add `--format` argument
- `crates/cgt-cli/src/main.rs` - Dispatch to appropriate formatter

### Test Files

- `tests/data/*.txt` - Expected plain text outputs

## Implementation Steps

### 1. Create cgt-formatter-plain crate

```toml
# crates/cgt-formatter-plain/Cargo.toml
[package]
name = "cgt-formatter-plain"
version = "0.1.0"
edition = "2024"

[dependencies]
cgt-core = { path = "../cgt-core" }
rust_decimal = "1"
chrono = "0.4"

[lints]
workspace = true
```

### 2. Implement formatter

```rust
// crates/cgt-formatter-plain/src/lib.rs
use cgt_core::{TaxReport, Transaction};

pub fn format(report: &TaxReport, transactions: &[Transaction]) -> String {
    let mut output = String::new();
    // Build SUMMARY section
    // Build TAX YEAR DETAILS section
    // Build TAX RETURN INFORMATION section
    // Build HOLDINGS section
    // Build TRANSACTIONS section
    // Build ASSET EVENTS section
    output
}
```

### 3. Add CLI format argument

```rust
// crates/cgt-cli/src/commands.rs
#[derive(ValueEnum, Clone, Default)]
pub enum OutputFormat {
    #[default]
    Plain,
    Json,
}

#[derive(Subcommand)]
pub enum Commands {
    Report {
        file: PathBuf,
        #[arg(long)]
        year: i32,
        #[arg(long, short, value_enum, default_value_t = OutputFormat::Plain)]
        format: OutputFormat,
    },
}
```

### 4. Dispatch in main.rs

```rust
Commands::Report { file, year, format } => {
    let content = fs::read_to_string(file)?;
    let transactions = parse_file(&content)?;
    let report = calculate(transactions.clone(), *year)?;

    match format {
        OutputFormat::Plain => {
            println!("{}", cgt_formatter_plain::format(&report, &transactions));
        }
        OutputFormat::Json => {
            println!("{}", serde_json::to_string_pretty(&report)?);
        }
    }
}
```

## Usage

```bash
# Plain text (default)
cgt report --year 2023 transactions.cgt

# Explicit plain text
cgt report --year 2023 --format plain transactions.cgt

# JSON output
cgt report --year 2023 --format json transactions.cgt
```

## Testing

```bash
# Run all tests
cargo test

# Run formatter tests specifically
cargo test -p cgt-formatter-plain
```

## Verification

Compare output against cgtcalc reference:

1. Run same input through both tools
2. Verify numerical values match exactly
3. Minor formatting differences (spacing) are acceptable
