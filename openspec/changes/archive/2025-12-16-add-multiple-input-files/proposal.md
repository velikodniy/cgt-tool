# Change: Support Multiple Input Files

## Why

Users with transactions spread across multiple `.cgt` files (e.g., separate files per tax year, per broker, or per account) currently need to manually concatenate them before processing. Supporting multiple input files directly in the CLI reduces friction and aligns with common Unix patterns for file handling.

## What Changes

- `parse` command accepts one or more files (positional arguments)
- `report` command accepts one or more files (positional arguments)
- When multiple files are provided, their contents are concatenated in order before parsing
- Transactions from all files are sorted by date (stable sort, maintaining order for same-date transactions)
- For PDF output with multiple inputs, default filename is `report.pdf`
- PDF output refuses to overwrite existing files unless `--output` explicitly specifies the path

## Impact

- Affected specs: `cli`
- Affected code: `crates/cgt-cli/src/commands.rs`, `crates/cgt-cli/src/main.rs`
- Documentation: `README.md` usage examples
