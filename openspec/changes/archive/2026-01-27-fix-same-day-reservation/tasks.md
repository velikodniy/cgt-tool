## 1. Core Implementation

- [x] 1.1 Add helper function to calculate same-day disposal quantity for a given date and ticker in `bed_and_breakfast.rs`
- [x] 1.2 Modify B&B matching loop to subtract same-day disposal reservation from available shares before matching
- [x] 1.3 Cap reservation at acquisition quantity to handle edge case where same-day sells exceed buys

## 2. Test Fixtures

- [x] 2.1 Create `tests/inputs/SameDayReservation.cgt` with scenario where B&B from earlier disposal competes with Same Day
- [x] 2.2 Create `tests/json/SameDayReservation.json` golden file with expected Same Day priority behavior
- [x] 2.3 Add unit test in `crates/cgt-core/tests/matching_tests.rs` for Same Day reservation priority

## 3. Documentation

- [x] 3.1 Update `docs/tax-rules.md` to clarify Same Day priority over B&B per TCGA92/S106A(9)

## 4. Verification

- [x] 4.1 Run `cargo fmt && cargo clippy && cargo test` to verify all tests pass
- [x] 4.2 Run cross-validation with `scripts/cross-validate.py` to confirm alignment with cgt-calc
- [x] 4.3 Update any existing golden files that change due to the fix
