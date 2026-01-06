# Change: Fix Same-Day Buy+Sell with Asset Events

## Why

The `calculate_remaining_at_event()` function in `acquisition_ledger.rs` uses FIFO matching to determine share quantities for asset event apportionment. This is incorrect - UK CGT rules per CG51560 (TCGA92/S105(1)) require Same Day → B&B → S104 matching order.

The bug only manifests when there's a same-day BUY and SELL followed by an asset event. The `WithAssetEventsSameDay.cgt` test documents this unsupported scenario.

## What Changes

- **FIX**: Update `calculate_remaining_at_event()` to simulate correct CGT matching rules instead of FIFO
- **ENABLE**: Test case `WithAssetEventsSameDay` with expected output

## Impact

- Affected specs: `cgt-calculation`
- Affected code: `crates/cgt-core/src/matcher/acquisition_ledger.rs` (~20 lines changed)
- Affected tests: `tests/inputs/WithAssetEventsSameDay.cgt` (uncomment), add expected outputs
