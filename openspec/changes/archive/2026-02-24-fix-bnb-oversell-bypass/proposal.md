## Why

B&B matching bypasses the oversell check. When a SELL has no backing holding (ledger + S104 pool = 0 shares), the B&B matcher still matches it to a future BUY within 30 days, bringing `remaining` to zero and silently passing the oversell guard. Per HMRC CG51560 and CG51590 Example 1, B&B determines cost basis for a valid disposal — it does not enable disposing of shares the taxpayer does not hold.

## What Changes

- **BREAKING**: Add a holding check before the matching cascade in `process_sell`. If `ledger_remaining(ticker) + pool_quantity(ticker) < sell_amount`, return an error. This rejects input files that sell shares not held, which were previously silently accepted via B&B.
- Fix `MultipleMatches.cgt` test fixture: the 2019-08-28 SELL of 10 shares with 0 holding is invalid. Rewrite the fixture to test all three matching rules with valid holdings.
- Update golden files (JSON, plain text) for the rewritten fixture.
- Fix the converter (`convert-to-raw.py`) to sort output by date with BUYs before SELLs on the same day, so cgt-calc cross-validation works for files where the .cgt source has SELL before BUY within a day (e.g., `CarryLoss.cgt`).

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `single-pass-matcher`: Add requirement that SELL must be backed by sufficient holding (ledger + S104 pool) before matching begins.
- `testing`: Update MultipleMatches scenario to use valid holdings.

## Impact

- `crates/cgt-core/src/matcher/mod.rs` — holding check in `process_sell`
- `tests/inputs/MultipleMatches.cgt` — rewritten fixture
- `tests/json/MultipleMatches.json` — regenerated golden file
- `tests/plain/MultipleMatches.txt` — regenerated golden file
- `scripts/convert-to-raw.py` — sort output by date, BUY before SELL within same day
- Cross-validation: CarryLoss and MultipleMatches discrepancies should be resolved or reduced
