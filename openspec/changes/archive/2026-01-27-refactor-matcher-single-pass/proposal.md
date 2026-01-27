## Why

The current `Matcher` implementation relies on a "simulate-then-process" architecture. To apply corporate actions (Dividends, Cap Returns) correctly, `AcquisitionLedger` currently simulates the entire matching history (Same Day → B&B → S104) to estimate holdings at the event date. This results in:

1. **Logic Duplication**: Matching rules are implemented twice: once in `matcher/*.rs` for the report, and again in `acquisition_ledger.rs` for the simulation.
2. **Inefficiency**: The simulation creates O(N²) complexity.
3. **Maintainability Risk**: Any change to matching rules must be applied in two places, risking divergence.

## What Changes

We will refactor the `Matcher` to use a **single-pass O(N)** algorithm:

1. **Chronological Processing**: Transactions will be processed strictly in order. Corporate actions will apply to the *current* state of the ledger, eliminating the need for simulation.
2. **Peek-Forward B&B**: When processing a Sell, the matcher will "peek" ahead (up to 30 days) to identify Bed & Breakfast matches.
3. **Future Consumption Tracking**: A `future_consumption` map will track shares from future Buys that have been claimed by past B&B Sells, preventing double-counting when the Buy is eventually processed.
4. **Code Removal**: The `build_ledgers` phase and `AcquisitionLedger::calculate_remaining_at_event` simulation logic will be deleted.
5. **Refactoring**: Clean up `Operation` deserialization and `TaxPeriod` constants in `models.rs` to be more idiomatic.

## Capabilities

### New Capabilities

- `single-pass-matcher`: Specification for the O(N) matching engine architecture, defining the rules for chronological processing, B&B lookahead, and future consumption tracking.

### Modified Capabilities

No user-facing functional requirements are changing. The change is purely architectural.

## Impact

- **Core Logic**: Complete rewrite of `crates/cgt-core/src/matcher/mod.rs` and `crates/cgt-core/src/matcher/acquisition_ledger.rs`.
- **Models**: Minor refactoring in `crates/cgt-core/src/models.rs`.
- **Performance**: Significant reduction in complexity for portfolios with long histories and corporate actions.
- **Verification**: Output must remain identical to current implementation for all existing test cases.
