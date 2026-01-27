## Why

Schwab JSON conversion currently fails on real awards exports for two related reasons: some awards records provide vesting FMV as `VestFairMarketValue` with a separate `VestDate` (not `FairMarketValuePrice`), and some non-vesting awards actions (e.g., Wire Transfer, Tax Withholding) have empty `TransactionDetails`. The parser treats these as hard JSON errors, which blocks conversion of otherwise valid transactions and prevents users from generating .cgt outputs.

## What Changes

- Parse Schwab awards records that use `VestFairMarketValue`/`VestDate` for vesting, falling back to `FairMarketValuePrice` when vest fields are absent.
- Use `VestDate` (when present) as the acquisition date for RSU vesting lookups instead of the parent transaction date.
- Distinguish awards `Action` types and allow non-vesting actions with empty `TransactionDetails` to be ignored for FMV lookup.
- Convert the current serde "missing field" JSON error into a domain error that points to the missing FMV for the relevant symbol/date when vesting data is required.
- Keep Schwab transaction parsing unchanged; no changes to CGT matching rules (Same Day, Bed & Breakfast, Section 104) per CG51560/CG51570/CG51580.
- No changes to CGT calculation logic or matching rules (Same Day, Bed & Breakfast, Section 104) per CG51560/CG51570/CG51580.

## Capabilities

### New Capabilities

- _None._

### Modified Capabilities

- `broker-conversion`: Update Schwab conversion requirements to ensure exported activity is translated into accurate trade and cost basis inputs.

## Impact

- `crates/cgt-converter` Schwab awards JSON parsing and lookup logic (including awards action classification)
- Test fixtures and golden outputs for Schwab conversion scenarios
- Documentation for RSU vesting FMV source
- Downstream CGT calculations remain unchanged; inputs become reliable

## Verification

- Add a regression fixture for awards JSON missing `FairMarketValuePrice` that currently triggers a JSON parsing error
- Add a regression fixture for awards actions (e.g., Wire Transfer) with empty `TransactionDetails` and confirm conversion succeeds
- Verify RSU vest conversions fail with a targeted `MissingFairMarketValue` error only when an FMV lookup is required
- Cross-validate converted .cgt outputs for representative Schwab examples; confirm no changes to matching outcomes (CG51560/CG51570/CG51580)
