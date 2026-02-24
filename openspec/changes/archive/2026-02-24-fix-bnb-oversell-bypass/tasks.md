## 1. Pre-cascade holding check

- [x] 1.1 Add holding check at the top of `process_sell` in `matcher/mod.rs`: compute `ledger.remaining_for_date(sell_date) + pool.quantity` and return `CgtError::InvalidTransaction` if sell amount exceeds it
- [x] 1.2 Add unit test: SELL with zero holding and a B&B-eligible future BUY returns error
- [x] 1.3 Add unit test: SELL with sufficient pool holding and a B&B-eligible future BUY succeeds
- [x] 1.4 Add unit test: SELL exceeding partial holding (e.g., sell 100 with 50 held) returns error with quantities in message
- [x] 1.5 Verify existing oversell tests still pass (`test_oversell_returns_error`, `test_sell_without_prior_acquisition_returns_error`)

## 2. Fix MultipleMatches test fixture

- [x] 2.1 Rewrite `MultipleMatches.cgt` so every SELL has sufficient backing holding, while still exercising Same Day, B&B, and S104 matching
- [x] 2.2 Regenerate `tests/json/MultipleMatches.json` golden file
- [x] 2.3 Regenerate `tests/plain/MultipleMatches.txt` golden file
- [x] 2.4 Run `cargo test` to verify golden file tests pass

## 3. Fix converter output sorting

- [x] 3.1 Sort `convert-to-raw.py` output by `(date, action)` with BUY before SELL on the same day
- [x] 3.2 Verify `CarryLoss.cgt` converted output has BUY before SELL per day

## 4. Cross-validation and final checks

- [x] 4.1 Run `cargo fmt && cargo clippy --all-targets --all-features`
- [x] 4.2 Run `cargo test` (all tests pass)
- [x] 4.3 Run `python3 scripts/cross-validate.py tests/inputs/*.cgt` and confirm CarryLoss and MultipleMatches discrepancies are resolved
