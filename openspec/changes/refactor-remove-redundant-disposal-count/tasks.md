## 1. Core Model Change

- [x] 1.1 Remove `disposal_count` field from `TaxYearSummary` struct in `crates/cgt-core/src/models.rs`
- [x] 1.2 Add `pub fn disposal_count(&self) -> u32` method to `TaxYearSummary`
- [x] 1.3 Implement custom `Serialize` for `TaxYearSummary` to include `disposal_count` in JSON output

## 2. Update Construction Sites

- [x] 2.1 Remove `disposal_count` field assignment in `build_tax_year_summary` in `crates/cgt-core/src/calculator.rs`
- [x] 2.2 Remove `disposal_count` field assignment in `build_all_tax_year_summaries` in `crates/cgt-core/src/calculator.rs`

## 3. Update Formatters

- [x] 3.1 Update `crates/cgt-formatter-plain/src/lib.rs` to call `year.disposal_count()` method
- [x] 3.2 Update `crates/cgt-formatter-pdf/src/lib.rs` to call `year.disposal_count()` method

## 4. Update Tests

- [x] 4.1 Remove `disposal_count` field from test struct literals in `crates/cgt-formatter-plain/tests/lib_tests.rs`
- [x] 4.2 Remove `disposal_count` field from test struct literals in `crates/cgt-formatter-pdf/src/lib.rs`

## 5. Verification

- [x] 5.1 Run `cargo fmt && cargo clippy && cargo test` and verify all pass
- [x] 5.2 Verify JSON golden files still match (no golden file changes needed since serialization preserved)
