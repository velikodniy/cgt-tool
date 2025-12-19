# Change: Fix CAPRETURN/DIVIDEND Cost Apportionment

## Why

The cost adjustment logic for CAPRETURN and DIVIDEND events incorrectly apportioned adjustments when the event applied to fewer shares than the total holdings. This caused over-apportionment of cost adjustments, leading to incorrect gain/loss calculations.

For example, if holding 20 shares and receiving a CAPRETURN on 10 shares for a total of -£10, the code would apply:

- Lot 1 (10 shares): -£10 × (10/10) = -£10
- Lot 2 (10 shares): -£10 × (10/10) = -£10
- **Total: -£20 instead of -£10**

This caused discrepancies of £2-3 in calculated gains compared to reference calculations.

## What Changes

- **MODIFIED**: Cost adjustment apportionment formula in `acquisition_ledger.rs`

  - Changed from: `adjustment × (lot_remaining / event_amount)`
  - Changed to: `adjustment × (lot_remaining / total_held)`
  - This correctly distributes adjustments based on each lot's proportion of total holdings

- **ADDED**: Additional test cases for CAPRETURN with partial holdings

- **MODIFIED**: Updated TAX_RULES.md with clearer explanation of cost apportionment for S104 pooling

## Impact

- Affected specs: `cgt-calculation`
- Affected code: `crates/cgt-core/src/matcher/acquisition_ledger.rs`
- Test outputs updated: All `tests/plain/*.txt` and `tests/json/*.json` regenerated
- Fixes discrepancies in: AssetEventsNotFullSale, AssetEventsNotFullSale2, BuySellAllBuyAgainCapitalReturn, WithAssetEvents, WithAssetEventsBB

## Verification

After this fix, all discrepant test cases now match reference values within rounding tolerance:

| Test Case                                 | Reference | Before Fix | After Fix | Status |
| ----------------------------------------- | --------- | ---------- | --------- | ------ |
| AssetEventsNotFullSale (2020/21)          | £57       | £59.33     | £57.33    | Fixed  |
| AssetEventsNotFullSale2 (2020/21)         | £253      | £253.33    | £253.33   | OK     |
| BuySellAllBuyAgainCapitalReturn (2019/20) | -£6       | -£5.79     | -£5.79    | OK     |
| WithAssetEvents (2019/20)                 | £417      | £417.11    | £417.11   | OK     |
| WithAssetEventsBB (2019/20)               | £265      | £265.10    | £265.10   | OK     |
