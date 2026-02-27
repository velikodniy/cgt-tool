## 1. Make `from_date` fallible

- [x] 1.1 Change `TaxPeriod::from_date` signature to `-> Result<Self, CgtError>` and call `new()` internally
- [x] 1.2 Update `build_single_tax_year_summary` in `calculator.rs` to propagate the `Result`
- [x] 1.3 Update `build_all_tax_year_summaries` in `calculator.rs` to propagate the `Result`

## 2. Update tests

- [x] 2.1 Update existing `test_tax_period_from_date` test to handle the new `Result` return type
- [x] 2.2 Add a test for a date outside the valid range returning `CgtError::InvalidTaxYear`

## 3. Verify

- [x] 3.1 Run `cargo fmt && cargo clippy && cargo test` and confirm all pass
