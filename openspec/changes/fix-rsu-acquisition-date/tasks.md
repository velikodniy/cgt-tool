# Tasks: Fix RSU Acquisition Date

## 1. Update Awards Module

- [x] 1.1 Add `AwardLookup` struct containing `fmv: Decimal` and `vest_date: NaiveDate`
- [x] 1.2 Modify `AwardsData::get_fmv()` to return `Result<AwardLookup, ConvertError>` instead of `Result<Decimal, ConvertError>`
- [x] 1.3 Update `get_fmv()` to return the vest date found during lookback (the date that matched, not the query date)
- [x] 1.4 Update unit tests for `get_fmv()` to verify vest date is returned correctly

## 2. Update Schwab Converter

- [x] 2.1 Update `Stock Plan Activity` handling to use `AwardLookup::vest_date` as the BUY transaction date
- [x] 2.2 Keep the FMV from `AwardLookup::fmv` for the price (unchanged behavior)
- [x] 2.3 Add integration test verifying vest date is used instead of settlement date

## 3. Documentation

- [x] 3.1 Add "RSU Acquisition Date" section to `TAX_RULES.md` explaining:
  - Vest date vs settlement date distinction
  - HMRC guidance references (CG14250, ERSM20192)
  - Why vest date is used for CGT acquisition
- [x] 3.2 Add example showing impact on Same Day matching

## 4. Validation

- [x] 4.1 Run `cargo test` to verify all tests pass
- [x] 4.2 Run `cargo clippy` to verify no warnings
- [x] 4.3 Run `openspec validate fix-rsu-acquisition-date --strict` to verify spec changes
