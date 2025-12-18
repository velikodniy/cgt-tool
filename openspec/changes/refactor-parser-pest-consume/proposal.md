# Change: Refactor parser to use pest_consume with semantic grammar

## Why

The current parser implementation uses `pest` directly with manual error handling and verbose parsing logic. The grammar uses intermediate wrapper rules (`buy_sell_args`, `dividend_args`, `capreturn_args`, `split_args`) and separate clause rules (`fees_clause`, `tax_clause`) that add complexity without semantic value. By migrating to `pest_consume` and restructuring the grammar to use semantic types, we can:

- Simplify parsing code through derive-based parsing with semantic methods
- Improve error handling with automatic `pest_consume` error conversions
- Make the grammar more semantic and self-documenting by embedding keywords in semantic types (e.g., `price = { "@" ~ money }`)
- Reduce code duplication and manual parser state management

## What Changes

- Add `pest_consume` dependency to `crates/cgt-core/Cargo.toml`
- Restructure grammar in `parser.pest` to use semantic types instead of intermediate wrappers:
  - Replace wrapper rules (`buy_sell_args`, etc.) with flat command structures
  - Create semantic money types: `price`, `total_value`, `fees`, `tax`, `ratio_value` that include their keywords/markers
  - Make `date` atomic using `@` prefix for better parsing
- Replace `#[derive(Parser)]` with `#[derive(pest_consume::Parser)]` on parser struct
- Remove manual error handling helpers (~90 lines): `from_pest_error`, `extract_found_token`, `format_expected_rules`, `format_rule_name`
- Add error conversion for `pest_consume::Error` to `CgtError`
- Replace `parse_file` function to use `pest_consume` parsing methods
- Implement `pest_consume` semantic parsing methods for all grammar rules
- Update all parser tests to verify behavior is preserved
- Update MCP server DSL documentation to reflect any syntax clarifications

## Impact

- **Affected specs**: dsl-parsing
- **Affected code**:
  - `crates/cgt-core/Cargo.toml` (add dependency)
  - `crates/cgt-core/src/parser.pest` (grammar restructure)
  - `crates/cgt-core/src/parser.rs` (complete refactor ~400 lines)
  - `crates/cgt-core/src/error.rs` (add pest_consume error conversion)
  - `crates/cgt-core/tests/parser_tests.rs` (verify tests still pass)
  - `crates/cgt-mcp/src/resources.rs` (DSL syntax documentation)
  - `crates/cgt-mcp/src/server.rs` (tool descriptions)
  - `README.md` (DSL syntax examples if needed)
- **Breaking changes**: None - DSL syntax remains identical for users
- **Performance**: Neutral to slight improvement due to more efficient parsing
- **Testing**: All existing parser tests must pass; existing .cgt fixture files remain valid
