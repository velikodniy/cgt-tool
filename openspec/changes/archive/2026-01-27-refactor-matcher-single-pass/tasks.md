## 1. Model cleanup

- [x] 1.1 Refactor `Operation` deserialization to be idiomatic (case handling via `serde`) and update related tests
- [x] 1.2 Extract `TaxPeriod` year bounds to constants and update usages/tests

## 2. Matcher single-pass refactor (cross-verify with cgtcalc)

- [x] 2.1 Implement single-pass chronological `Matcher::process` with `future_consumption` tracking and remove `build_ledgers`
- [x] 2.2 Remove `AcquisitionLedger::calculate_remaining_at_event` and adjust cost adjustments to use live holdings state

## 3. Matching strategies integration (cross-verify with cgtcalc)

- [x] 3.1 Implement peek-forward B&B matching with split/unsplit ratio tracking in the 30-day window
- [x] 3.2 Update same-day and Section 104 matching to work with new consumption tracking

## 4. Tests and verification (cross-verify with cgtcalc)

- [x] 4.1 Add golden fixtures for the new scenarios in `single-pass-matcher` spec
- [x] 4.2 Run `cargo fmt && cargo clippy && cargo test` and `python3 scripts/cross-validate.py tests/inputs/*.cgt`
