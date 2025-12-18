# Change: Make year parameter optional for all-years reports

## Why

Enable compatibility with cgtcalc test outputs by allowing reports to include all tax years with disposals. Currently, the `--year` parameter is required, forcing users to generate separate reports per year. The cgtcalc tool outputs all years by default, making test comparison difficult without modification.

## What Changes

- **MODIFIED**: CLI `--year` parameter becomes optional
- **MODIFIED**: When `--year` is omitted, `calculate_report` returns all tax years with disposals
- **MODIFIED**: Calculator accepts `Option<i32>` instead of requiring a year
- **MODIFIED**: MCP `calculate_report` tool's `year` parameter becomes optional

## Impact

- Affected specs: `cli`, `cgt-calculation`, `mcp-server`
- Affected code:
  - `crates/cgt-cli/src/commands.rs` (CLI argument definition)
  - `crates/cgt-cli/src/main.rs` (command handling)
  - `crates/cgt-core/src/calculator.rs` (core calculation logic)
  - `crates/cgt-mcp/src/server.rs` (MCP tool schema and handler)
- Output format: No changes to report structure; `TaxReport.tax_years` already supports multiple years
