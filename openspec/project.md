# Project Context

## Purpose

CLI tool for UK Capital Gains Tax calculation. Implements HMRC share matching rules (Same Day, Bed & Breakfast, Section 104 pooling) for share transactions. Parses custom DSL input, outputs reports in plain text, JSON, or PDF.

## Tech Stack

- **Language**: Rust 2024 edition
- **Crates**:
  - `cgt-core` — Parsing, calculation engine, data model
  - `cgt-cli` — CLI binary `cgt-tool` (clap)
  - `cgt-formatter-plain` — Plain text reports
  - `cgt-formatter-pdf` — PDF reports (typst-as-lib)
  - `cgt-fx` — FX conversion (HMRC rates)
- **Key dependencies**: `pest` (parsing), `rust_decimal` (money), `chrono` (dates), `thiserror`/`anyhow` (errors), `serde` (serialization)

## Architecture

- **Workspace structure**: Separate crates for core logic, CLI, formatters
- **DSL-driven input**: Custom grammar in `cgt-core/src/parser.pest`
- **IO-free core**: Calculation logic is IO-free, WASM-friendly
- **Bundled FX rates**: HMRC rates embedded at compile time; runtime override via `--fx-folder`

## Testing

- **Fixtures**: `tests/inputs/*.cgt`, expected outputs in `tests/json/` and `tests/plain/`
- **Golden file testing**: Compare actual vs expected output
- **Integration tests**: `crates/cgt-core/tests/` and `crates/cgt-cli/tests/`

## Git Workflow

- **Pre-commit hooks**: `.pre-commit-config.yaml` (clippy, fmt, trailing whitespace)
- **Branches**: Named after capability or change (e.g., `add-rate-split`)

## External Dependencies

- **HMRC Exchange Rates**: Monthly XML from gov.uk
  - 2021+: `https://www.trade-tariff.service.gov.uk/api/v2/exchange_rates/files/monthly_xml_YYYY-MM.xml`
  - Pre-2021: National Archives
- **Download script**: `scripts/download-fx-rates.sh`
