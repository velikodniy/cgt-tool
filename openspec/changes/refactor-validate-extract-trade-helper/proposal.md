## Why

The `validate()` function in `crates/cgt-core/src/validation.rs` contains ~150 lines of near-identical validation checks (zero quantity, negative quantity, negative price, negative fees) copy-pasted across the Buy, Sell, and CapReturn match arms. This duplication makes the function harder to maintain and increases the risk of inconsistencies when adding new validations.

## What Changes

- Extract a `check_trade_fields()` helper function that performs the four common field validations (zero quantity, negative quantity, negative price, negative fees)
- Replace duplicated validation blocks in Buy, Sell, and CapReturn arms with calls to this helper
- Retain operation-specific validations (e.g., sell-before-buy warning) in their respective match arms
- No change to validation behavior — purely structural refactor

## Capabilities

### New Capabilities

(none — this is a refactor with no new capabilities)

### Modified Capabilities

(none — no requirement-level changes, only implementation structure)

## Impact

- `crates/cgt-core/src/validation.rs`: Primary file affected. Internal restructuring only; public API (`validate()`, `ValidationResult`, etc.) unchanged.
- All existing tests must continue to pass without modification.

## Verification

Correctness is validated by ensuring all existing tests pass (`cargo test`), confirming identical validation behavior before and after the refactor. No tax calculation logic is affected.
