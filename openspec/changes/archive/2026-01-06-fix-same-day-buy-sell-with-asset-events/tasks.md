# Tasks: Fix Same-Day Buy+Sell with Asset Events

## 1. Fix the Bug

- [x] 1.1 Update `calculate_remaining_at_event()` in `acquisition_ledger.rs` to use correct CGT matching order (Same Day → B&B → S104) instead of FIFO

## 2. Enable Test Case

- [x] 2.1 Uncomment transactions in `tests/inputs/WithAssetEventsSameDay.cgt`
- [x] 2.2 Calculate expected results manually
- [x] 2.3 Create `tests/json/WithAssetEventsSameDay.json`
- [x] 2.4 Update `tests/plain/WithAssetEventsSameDay.txt`

## 3. Update Documentation

- [x] 3.1 Add TCGA92/S105(1)(a) reference to TAX_RULES.md Same Day Rule section
- [x] 3.2 Add example showing same-day buy+sell with subsequent asset event

## 4. Verify

- [x] 4.1 Run `cargo test` - all tests pass
- [x] 4.2 Run `cargo clippy` - no warnings
