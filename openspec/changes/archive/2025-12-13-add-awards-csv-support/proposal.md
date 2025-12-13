# Change: Add CSV Format Support for Schwab Equity Awards

## Why

Schwab provides equity awards data in both JSON and CSV formats. Currently, the converter only supports JSON format for awards files. Users who export awards data as CSV must manually convert it to JSON, which is cumbersome and error-prone. Supporting CSV format directly simplifies the workflow and reduces manual data transformation.

## What Changes

- **UPDATE**: Awards parser to detect format based on file extension (`.json` vs `.csv`)
- **NEW**: CSV parser for Schwab equity awards with paired-row format support
- **UPDATE**: `parse_awards_json` function renamed to generic `parse_awards` that dispatches to format-specific parsers
- **NEW**: Tests for CSV awards parsing including paired-row format
- **UPDATE**: CLI documentation to indicate both JSON and CSV formats are supported

## Impact

- Affected specs: `broker-conversion` (MODIFIED requirements for awards parsing)
- Affected code:
  - `crates/cgt-converter/src/schwab/awards.rs` — add CSV parser, format detection
  - `crates/cgt-converter/src/schwab/mod.rs` — update function call
  - `crates/cgt-converter/tests/` — add CSV awards test fixtures
  - `README.md` — update documentation
