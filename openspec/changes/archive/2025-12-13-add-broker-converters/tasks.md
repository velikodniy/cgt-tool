# Tasks: Add Broker Export Converters

## 1. Core Infrastructure

- [x] 1.1 Create `crates/cgt-converter/` crate with Cargo.toml
- [x] 1.2 Define `BrokerConverter` trait with associated `Input` type in `src/lib.rs`
- [x] 1.3 Define `ConvertOutput` struct with content, warnings, skipped count
- [x] 1.4 Define `ConvertError` error types in `src/error.rs`
- [x] 1.5 Create `src/output.rs` with CGT DSL formatting utilities

## 2. Schwab Transactions Parser

- [x] 2.1 Create `src/schwab/mod.rs` module structure
- [x] 2.2 Implement CSV row parsing with column validation
- [x] 2.3 Implement date parsing (MM/DD/YYYY, "as of" handling)
- [x] 2.4 Implement action type mapping (Buy→BUY, Sell→SELL, etc.)
- [x] 2.5 Implement currency/amount parsing ($-prefixed values)
- [x] 2.6 Handle skipped transaction types (Wire, Interest, etc.)

## 3. Schwab Awards Parser

- [x] 3.1 Create `src/schwab/awards.rs` for equity awards JSON
- [x] 3.2 Define JSON schema structs with serde deserialization
- [x] 3.3 Parse Fair Market Value prices by date/symbol
- [x] 3.4 Implement FMV lookup for Stock Plan Activity transactions
- [x] 3.5 Handle date matching with 7-day lookback (per KapJI implementation)

## 4. Schwab Converter Integration

- [x] 4.1 Implement `BrokerConverter` for `SchwabConverter`
- [x] 4.2 Merge transactions with award prices
- [x] 4.3 Handle dividend + tax withholding combination
- [x] 4.4 Sort transactions chronologically (oldest first)
- [x] 4.5 Generate header comments with metadata
- [x] 4.6 Generate inline comments for RSU vesting, "as of" dates

## 5. Test Fixtures

- [x] 5.1 Create test fixtures from KapJI/schwab examples
- [x] 5.2 Add basic buy/sell conversion test
- [x] 5.3 Add RSU vesting with awards file test
- [x] 5.4 Add dividend with tax withholding test
- [x] 5.5 Add date format variations test
- [x] 5.6 Add error case tests (missing columns, invalid dates)
- [x] 5.7 Add end-to-end conversion test with expected output

## 6. CLI Integration

- [x] 6.1 Add `convert` subcommand to `cgt-cli` with broker subcommands
- [x] 6.2 Add `schwab` subcommand with positional transactions file arg
- [x] 6.3 Add `--awards` optional flag for Schwab awards JSON
- [x] 6.4 Add `-o`/`--output` for file output (default stdout)
- [x] 6.5 Add error handling for file read failures
- [x] 6.6 Add integration test for CLI convert command

## 7. Documentation

- [x] 7.1 Update README.md with converter usage
- [x] 7.2 Add Schwab export instructions (how to download CSV)
- [x] 7.3 Document supported/unsupported transaction types
- [x] 7.4 Add example conversion workflow

## 8. Validation & Cleanup

- [x] 8.1 Run `cargo clippy` — fix all warnings
- [x] 8.2 Run `cargo fmt` — format code
- [x] 8.3 Run full test suite
- [x] 8.4 Verify WASM compatibility (no std::fs usage in converter)
