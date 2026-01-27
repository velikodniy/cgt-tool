## 1. Awards Parsing Updates

- [x] 1.1 Update Schwab awards JSON structs to accept optional `VestDate` and `VestFairMarketValue` alongside `FairMarketValuePrice`.
- [x] 1.2 Adjust awards parsing to extract FMV and vest date from vesting fields when present, falling back to `FairMarketValuePrice` + parent `Date` otherwise.
- [x] 1.3 Defer FMV validation to lookup time and return `MissingFairMarketValue` with symbol/date context when no FMV is available.
- [x] 1.4 Ensure parsing remains IO-free and uses Decimal for all price fields; keep symbol normalization case-insensitive.

## 2. Converter Integration

- [x] 2.1 Wire updated awards lookup into Schwab `Stock Plan Activity` conversion without changing transaction parsing or matching logic.
- [x] 2.2 Preserve 7-day lookback behavior while returning matched vest date (not settlement date).

## 3. Tests & Fixtures

- [x] 3.1 Add a regression awards JSON fixture where vesting records use `VestDate`/`VestFairMarketValue` and omit `FairMarketValuePrice`.
- [x] 3.2 Add unit tests for awards parsing to verify vest FMV selection, vest date override, and `MissingFairMarketValue` error path.
- [x] 3.3 Add/adjust integration test for Schwab RSU conversion using vest-date FMV from awards file.
- [x] 3.4 Verify golden outputs remain unchanged; add new fixtures/tests instead of modifying existing expected numbers.

## 4. Documentation & Validation

- [x] 4.1 Update `docs/tax-rules.md` to explicitly mention vest FMV source for RSU acquisition (ERSM20192) alongside vest date guidance.
- [x] 4.2 Run `cargo fmt && cargo clippy && cargo test` after each stage and ensure all tests pass.

## 5. Awards Action Modeling

- [x] 5.1 Add an awards `Action` enum and parse the `Action` field from Schwab awards JSON.
- [x] 5.2 Allow empty `TransactionDetails` only for non-vesting actions (Wire Transfer, Tax Withholding, Tax Reversal, Forced Disbursement); treat empty details as an error for unknown or vesting actions.
- [x] 5.3 Add wire transfer awards parsing tests (empty details accepted) and ensure missing details for unknown actions fails.
- [x] 5.4 Update OpenSpec specs/docs to reflect awards action handling.
