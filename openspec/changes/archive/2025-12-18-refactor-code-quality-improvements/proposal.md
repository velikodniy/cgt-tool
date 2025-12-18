# Change: Refactor Code Quality Improvements

## Why

Code review identified several robustness and maintainability issues: potential division by zero, symbol case inconsistency, missing CSV column validation, unclear validation contract, inflexible global config, oversized server file with embedded strings, unused error variants, and inconsistent coding patterns.

## What Changes

1. **Division safety** - Add defensive guards in Section 104 pool matching to prevent division by zero when `total_sell_amount` is zero
2. **Symbol case normalization** - Uppercase symbols in awards file lookup to match the ticker normalization used elsewhere
3. **CSV column validation** - Add upfront validation of required columns in Schwab transaction CSV parser with clear error messages
4. **Validation contract** - Document that `validate()` must be called before `calculate()` or integrate validation into the calculation pipeline
5. **Config flexibility** - Replace `OnceLock` global config with explicit parameter passing for testability; consolidate exemption tests
6. **MCP server size** - Extract string constants and templates from `server.rs` to `resources.rs` or dedicated template files
7. **Unused error variants** - Remove `InvalidParameter`, `ResourceNotFound`, and `DisposalNotFound` from `McpServerError`
8. **Consistent let-else** - Standardize on `let-else` pattern where applicable in matcher modules

## Impact

- Affected specs: `cgt-calculation`, `broker-conversion`, `mcp-server`
- Affected code:
  - `crates/cgt-core/src/matcher/section104.rs`
  - `crates/cgt-core/src/matcher/same_day.rs`
  - `crates/cgt-core/src/matcher/bed_and_breakfast.rs`
  - `crates/cgt-core/src/calculator.rs`
  - `crates/cgt-core/src/exemption.rs`
  - `crates/cgt-core/src/config.rs`
  - `crates/cgt-converter/src/schwab/awards.rs`
  - `crates/cgt-converter/src/schwab/mod.rs`
  - `crates/cgt-mcp/src/server.rs`
  - `crates/cgt-mcp/src/resources.rs`
  - `crates/cgt-mcp/src/error.rs`
