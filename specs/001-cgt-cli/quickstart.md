# Quickstart: CGT CLI Tool

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
./target/release/cgt-cli parse transactions.txt

# Generate JSON Schema
./target/release/cgt-cli parse --schema
```

### 2. Generate Tax Report

```bash
# Generate full tax report
./target/release/cgt-cli report transactions.txt > report.json
```

## Input File Format (DSL)

One transaction per line. The parser supports flexible whitespace between elements and comments starting with `#`. Format: `YYYY-MM-DD ACTION TICKER AMOUNT @ PRICE [EXPENSES EXPENSE_AMOUNT]`

```text
# Example transactions
2025-04-01 BUY AAPL 100 @ 150.00 EXPENSES 5.00
2025-04-01 BUY AAPL 50 @ 155.00 # Expenses are optional
2025-05-01 SELL AAPL 50 @ 160.00 EXPENSES 5.00
```

- **Actions**: `BUY`, `SELL`, `DIVIDEND`, `CAPRETURN`, `SPLIT`, `UNSPLIT`
- **BUY/SELL**: `DATE ACTION TICKER AMOUNT @ PRICE [EXPENSES EXPENSE_AMOUNT]`
- **DIVIDEND**: `DATE DIVIDEND TICKER AMOUNT TAX_PAID`
- **CAPRETURN**: `DATE CAPRETURN TICKER AMOUNT EXPENSES`
- **SPLIT/UNSPLIT**: `DATE SPLIT TICKER RATIO`

## Development

### Project Structure

- `crates/cgt-core`: The logic engine (Parser + Calculator). No CLI code here.
- `crates/cgt-cli`: The command-line interface.

### Adding a new Transaction Type

1. Update `Action` enum in `crates/cgt-core/src/models.rs`.
2. Update `parser.pest` and `parser.rs` in `cgt-core`.
3. Handle the new action in `crates/cgt-core/src/calculator.rs`.
