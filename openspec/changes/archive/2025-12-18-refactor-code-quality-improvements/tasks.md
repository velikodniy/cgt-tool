# Tasks

## 1. Division Safety in Section 104 Matching

- [x] 1.1 Add guard in `section104.rs` to return early if `total_sell_amount` is zero before division on line 50
- [x] 1.2 Add unit test for zero sell amount edge case
- [x] 1.3 Review `same_day.rs` and `bed_and_breakfast.rs` for similar division patterns and add guards if needed

## 2. Symbol Case Normalization in Awards Lookup

- [x] 2.1 Uppercase symbol in `AwardsData::get_fmv()` before HashMap lookup
- [x] 2.2 Update existing tests to verify case-insensitive behavior
- [x] 2.3 Add explicit test for lowercase symbol lookup succeeding

## 3. CSV Extra Column Tolerance in Schwab Converter

- [x] 3.1 Verify `flexible(true)` is set on CSV reader (already done)
- [x] 3.2 Add test case for CSV with extra empty columns to confirm tolerance
- [x] 3.3 Document in code comments that extra columns are intentionally ignored

## 4. Validation Contract Clarification

- [x] 4.1 Add doc comment to `calculate()` stating that validation should be called first
- [x] 4.2 Consider adding optional `validate_first` parameter or separate `calculate_validated()` entry point
- [x] 4.3 Update `cgt-calculation` spec to document the validation requirement

## 5. Config Flexibility and Test Consolidation

- [x] 5.1 Add `get_exemption_with_config(year, config: &Config)` function that takes explicit config
- [x] 5.2 Update `get_exemption()` to call the new function with global config
- [x] 5.3 Consolidate exemption tests: replace 11 individual year tests with a single parameterized test
- [x] 5.4 Keep boundary tests (unsupported past/future) as separate tests

## 6. Extract MCP Server Strings to Resources

- [x] 6.1 Move hint constants (`HINT_*`) from `server.rs` to `resources.rs`
- [x] 6.2 Move `DSL_SYNTAX_REFERENCE` constant to `resources.rs`
- [x] 6.3 Move `EXAMPLE_TRANSACTION` constant to `resources.rs`
- [x] 6.4 Update `server.rs` imports to use resources module
- [x] 6.5 Verify server.rs is noticeably smaller (target: ~700-800 lines excluding tests)

## 7. Remove Unused Error Variants

- [x] 7.1 Remove `InvalidParameter` variant from `McpServerError`
- [x] 7.2 Remove `ResourceNotFound` variant from `McpServerError`
- [x] 7.3 Remove `DisposalNotFound` variant from `McpServerError`
- [x] 7.4 Run `cargo build` to confirm no compilation errors
- [x] 7.5 Run `cargo clippy` to confirm no dead code warnings

## 8. Standardize let-else Pattern

- [x] 8.1 Convert `match ... { Some(x) => x, None => return }` to `let Some(x) = ... else { return }` in `same_day.rs`
- [x] 8.2 Convert similar patterns in `section104.rs`
- [x] 8.3 Convert similar patterns in `bed_and_breakfast.rs`
- [x] 8.4 Run `cargo fmt` and `cargo clippy` to verify style consistency

## 9. Final Verification

- [x] 9.1 Run full test suite: `cargo test`
- [x] 9.2 Run clippy with all checks: `cargo clippy`
- [x] 9.3 Verify no regressions in existing functionality
