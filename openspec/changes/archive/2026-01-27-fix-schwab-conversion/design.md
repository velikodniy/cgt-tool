## Context

Schwab broker conversion currently fails on real awards exports because some vesting records provide FMV as `VestFairMarketValue` with a distinct `VestDate` (omitting `FairMarketValuePrice`), and some non-vesting awards actions (e.g., Wire Transfer, Tax Withholding) contain empty `TransactionDetails`. The awards parser treats both as hard errors, which aborts conversion. The conversion logic lives in `crates/cgt-converter` and feeds core CGT calculations, so resilient parsing and correct date/FM V mapping are critical. No changes are planned for matching rules or tax logic.

## Goals / Non-Goals

**Goals:**

- Parse Schwab awards JSON even when some records omit `FairMarketValuePrice`.
- Use `VestDate` and `VestFairMarketValue` when present for RSU vesting lookups; fall back to `FairMarketValuePrice` with the parent transaction date otherwise.
- Return a targeted domain error only when an RSU vest lookup requires FMV and none is available for that symbol/date.
- Preserve current transaction conversion and CGT matching behavior.
- Allow non-vesting awards actions with empty `TransactionDetails` to be ignored for FMV lookup without aborting conversion.

**Non-Goals:**

- Changing CGT calculation logic or HMRC matching rules.
- Adding new broker formats beyond existing Schwab JSON inputs.
- Redesigning the overall converter architecture.

## Decisions

- **Capture vesting-specific fields (`VestDate`, `VestFairMarketValue`) when present** and use them as the FMV source and acquisition date for RSU vesting lookups.
  - *Alternative:* Ignore vesting fields and rely on `FairMarketValuePrice` + parent date. Rejected because real exports omit `FairMarketValuePrice` and vest date differs.
- **Make `FairMarketValuePrice` optional in awards parsing** and capture FMV as `Option<Decimal>` per award detail record, selecting the correct field by context.
  - *Alternative:* Keep strict serde requirement and fail during JSON parse. Rejected because it blocks valid files and hides which RSU lookup is missing FMV.
- **Defer FMV validation to lookup time** so the converter can parse the awards file fully and only error when a specific RSU vest needs FMV.
  - *Alternative:* Fail the entire awards parse if any record lacks FMV. Rejected because missing FMV in unrelated records should not prevent conversion.
- **Emit `MissingFairMarketValue` with symbol/date context** when lookup fails, rather than a generic JSON parsing error.
  - *Alternative:* Return a generic parse error. Rejected due to poor debuggability.
- **Parse awards `Action` into a typed enum and classify FMV-relevant actions** (e.g., Deposit, Lapse, Sale, Forced Quick Sell) versus non-vesting cash actions (e.g., Wire Transfer, Tax Withholding).
  - *Alternative:* Ignore action types and infer solely from detail fields. Rejected because empty-detail actions cause false errors and we need to avoid skipping vesting data.
- **Ignore empty `TransactionDetails` only for non-vesting actions**; treat empty details as an error for unknown or FMV-relevant actions.
  - *Alternative:* Skip all empty-detail actions. Rejected because it can hide missing vesting data.

## Risks / Trade-offs

- [Risk] VestDate differs from parent transaction Date, affecting acquisition date → Mitigation: always prefer VestDate when present and add regression tests for vest-date overrides.
- [Risk] Missing FMV entries cause late errors during conversion → Mitigation: include symbol/date in error and add a regression test for the missing-FMV awards fixture.
- [Risk] Optional FMV could mask truly malformed awards records → Mitigation: treat missing FMV as an error when an RSU vest lookup references the record.
- [Risk] New action classification might mislabel unknown awards actions → Mitigation: fail fast on unknown action with empty details and add test coverage for known non-vesting actions.
