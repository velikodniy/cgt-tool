## Why

`Transaction::to_gbp` contains ~70 lines of repetitive match arms where Buy/Sell/Dividend/CapReturn each perform identical `amount_to_gbp` conversion logic on their monetary fields. This duplication makes maintenance harder and increases the risk of inconsistency when adding new variants or modifying conversion logic.

## What Changes

- Add a `to_gbp()` method on `Operation<CurrencyAmount>` that encapsulates the per-variant currency conversion logic
- Simplify `Transaction::to_gbp` to delegate to `self.operation.to_gbp(...)` instead of inlining the match
- No behavioral changes; purely structural refactoring

## Capabilities

### New Capabilities

(none)

### Modified Capabilities

(none -- this is a pure internal refactor with no requirement-level changes)

## Impact

- `crates/cgt-core/src/models.rs`: The only file affected. `Transaction::to_gbp` shrinks from ~70 lines to ~6 lines, and `Operation<CurrencyAmount>` gains a `to_gbp` method.
- No public API changes, no test changes expected.

## Verification

All existing tests (`cargo test`) and golden-file comparisons must continue to pass without modification, confirming behavior is preserved.
