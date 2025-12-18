# Implementation Tasks

## 1. Dependency Setup

- [x] 1.1 Add `pest_consume = "1.1"` to `crates/cgt-core/Cargo.toml` (version corrected from 2.7)
- [x] 1.2 Verify dependency resolves with `cargo build -p cgt-core`

## 2. Grammar Refactor

- [x] 2.1 Update `crates/cgt-core/src/parser.pest` with semantic types
  - [x] Create semantic money types: `price`, `total_value`, `fees`, `tax`, `ratio_value`
  - [x] Make `date` atomic using `@` prefix
  - [x] Remove intermediate wrapper rules: `buy_sell_args`, `dividend_args`, `capreturn_args`, `split_args`
  - [x] Remove separate clause rules: `fees_clause`, `tax_clause`
  - [x] Flatten command rules to use semantic types directly
- [x] 2.2 Test grammar compiles: `cargo build -p cgt-core`

## 3. Parser Implementation

- [x] 3.1 Update parser struct in `crates/cgt-core/src/parser.rs`
  - [x] Replace `#[derive(Parser)]` with `#[pest_consume::Parser]`
  - [x] Add `#[grammar = "parser.pest"]` attribute
  - [x] Define type aliases: `ParseResult<T>`, `Node<'i>`
- [x] 3.2 Implement semantic parsing methods using `pest_consume`
  - [x] `transaction_list(input) -> Vec<Transaction>`
  - [x] `transaction(input) -> Transaction`
  - [x] `command(input) -> (String, Operation)`
  - [x] `cmd_buy(input)`, `cmd_sell(input)`, `cmd_dividend(input)`, `cmd_capreturn(input)`, `cmd_split(input)`, `cmd_unsplit(input)`
  - [x] `money(input) -> CurrencyAmount`
  - [x] `price(input) -> CurrencyAmount`
  - [x] `total_value(input) -> CurrencyAmount`
  - [x] `fees(input) -> CurrencyAmount`
  - [x] `tax(input) -> CurrencyAmount`
  - [x] `ratio_value(input) -> Decimal`
  - [x] Helper methods for parsing `date`, `ticker`, `quantity`, `decimal`
- [x] 3.3 Replace `parse_file` function to use `pest_consume` entry point
- [x] 3.4 Remove old helper functions (lines 15-105):
  - [x] Delete `from_pest_error`
  - [x] Delete `extract_found_token`
  - [x] Delete `format_expected_rules`
  - [x] Delete `format_rule_name`

## 4. Error Handling

- [x] 4.1 Add error conversion in `crates/cgt-core/src/error.rs`
  - [x] Implement `From<pest_consume::Error<Rule>>` for `CgtError`
  - [x] Ensure error messages include line/column information
  - [x] Preserve error context for debugging
- [x] 4.2 Update `ParseError` variant if needed to accommodate pest_consume errors

## 5. Testing

- [x] 5.1 Run existing parser tests: `cargo test -p cgt-core parser_tests`
- [x] 5.2 Verify all parser tests pass without modification
- [x] 5.3 Run integration tests with fixture files: `cargo test -p cgt-core matching_tests`
- [x] 5.4 Test error cases explicitly:
  - [x] Invalid date format
  - [x] Missing required field
  - [x] Invalid currency code
  - [x] Malformed transaction
- [x] 5.5 Run full test suite: `cargo test`
- [x] 5.6 Verify all ~30 .cgt fixture files in `tests/inputs/` parse correctly
- [x] 5.7 Run clippy: `cargo clippy -p cgt-core` (one expected warning about large error type)
- [x] 5.8 Run formatter: `cargo fmt`

## 6. Documentation Updates

- [x] 6.1 Update `crates/cgt-mcp/src/resources.rs` DSL_SYNTAX if needed (no changes required)
- [x] 6.2 Update `crates/cgt-mcp/src/server.rs` tool descriptions if needed (updated error message check)
- [x] 6.3 Review `README.md` for any parser-specific documentation (no changes required)
- [x] 6.4 Update `AGENTS.md` if parser development guidelines change (no changes required)

## 7. Validation

- [x] 7.1 Verify no regression in error message quality
- [x] 7.2 Confirm performance is similar or better (spot check with large files)
- [x] 7.3 Ensure all existing DSL syntax remains valid
- [x] 7.4 Run end-to-end CLI tests: `cargo test -p cgt-cli`

## Notes

- Corrected `pest_consume` version from 2.7 to 1.1 (latest available)
- One clippy warning about large error type size is expected and acceptable (external type)
- Updated MCP test to check for "Parsing error" instead of "Parse error"
- All 275+ tests pass successfully
- DSL syntax unchanged - full backward compatibility maintained
