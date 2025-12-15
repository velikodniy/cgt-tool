# Change: Fix RSU Acquisition Date for CGT Matching

## Why

RSU (Restricted Stock Unit) acquisitions are currently recorded using the **settlement date** from Schwab's "Stock Plan Activity" transaction (the date shares appear in the brokerage account). However, per HMRC guidance (CG14250, ERSM20192), the CGT acquisition date should be the **vest date** (when conditions are satisfied and ownership becomes unconditional) - not the settlement date which is typically 2-3 days later due to T+2 settlement.

This date difference affects CGT matching rules:

- **Same Day Rule**: A sale on vest date should match acquisitions on the same vest date
- **Bed & Breakfast Rule**: The 30-day window should be calculated from the vest date

Using settlement dates instead of vest dates can cause:

1. Same-day matches to be missed (acquisition appears 2+ days after disposal)
2. B&B matches to use incorrect cost basis
3. Different CGT calculations compared to HMRC expectations

## What Changes

- **Schwab Awards Parser**: Return vest date alongside FMV when looking up awards
- **Schwab Converter**: Use vest date (from awards file) as acquisition date for `Stock Plan Activity` transactions, not the settlement date from transactions CSV
- **TAX_RULES.md**: Add section documenting RSU acquisition date treatment per HMRC guidance
- **Tests**: Update to verify vest date is used for acquisition matching

## Impact

- Affected specs: `broker-conversion`
- Affected code: `crates/cgt-converter/src/schwab/awards.rs`, `crates/cgt-converter/src/schwab/mod.rs`
- Affected docs: `TAX_RULES.md`

## HMRC References

- **CG14250**: "If the contract is conditional the date of disposal is the date all of the conditions are satisfied"
- **ERSM20192**: "An RSU award will vest when all the conditions laid down to be satisfied before the stock or shares may be issued have been met"
- **CG51560**: Same Day and B&B rules use acquisition dates for matching
