# Quickstart: DSL Enhancements

## Prerequisites

- **Rust**: Latest stable version (`rustup update`)
- **Cargo**: Included with Rust

## Building

```bash
# Build the workspace
cargo build --release

# Run tests
cargo test
```

## Running the CLI

The binary is located at `target/release/cgt-cli`.

### 1. Parse a file (Debug/Verify)

```bash
# Output parsed transactions as JSON
./target/release/cgt-cli parse transactions.cgt

# Generate JSON Schema
./target/release/cgt-cli parse --schema
```

### 2. Generate Tax Report

```bash
# Generate full tax report
./target/release/cgt-cli report transactions.cgt --year 2023
```

## Input File Format (DSL)

One transaction per line. The parser supports flexible whitespace between elements and comments starting with `#`.

**New Syntax Examples:**

- **DIVIDEND**: `YYYY-MM-DD DIVIDEND TICKER AMOUNT TAX TAX_AMOUNT`
  (e.g., `2019-11-30 DIVIDEND GB00B3TYHH97 110.93 TAX 0`)
- **CAPRETURN**: `YYYY-MM-DD CAPRETURN TICKER AMOUNT EXPENSES EXPENSE_AMOUNT`
  (e.g., `2019-05-31 CAPRETURN GB00B3TYHH97 149.75 EXPENSES 0`)
- **SPLIT/UNSPLIT**: `YYYY-MM-DD SPLIT FOO RATIO RATIO_VALUE`
  (e.g., `2019-02-15 SPLIT FOO RATIO 2`)

## Development

### Project Structure

- `crates/cgt-core`: The logic engine (Parser + Calculator).
- `crates/cgt-cli`: The command-line interface.

### Adding a new Transaction Type

1. Update `Action` enum in `crates/cgt-core/src/models.rs`.
2. Update `parser.pest` and `parser.rs` in `cgt-core`.
3. Handle the new action in `crates/cgt-core/src/calculator.rs`.
