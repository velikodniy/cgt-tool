## Why

`AcquisitionLot::remaining_amount` is set to `original_amount` in the constructor and never modified anywhere in the codebase. The `available()` and `held_for_adjustment()` methods read it as if it could differ from `original_amount`, but it cannot. This dead field is misleading and adds unnecessary state to a correctness-critical data structure.

## What Changes

- Remove the `remaining_amount` field from `AcquisitionLot`
- Replace all references to `remaining_amount` with `original_amount` in `available()` and `held_for_adjustment()`
- No behavioral change — this is a pure refactor eliminating dead state

## Capabilities

### New Capabilities

(none)

### Modified Capabilities

(none — this is an internal refactor with no spec-level behavior change)

## Impact

- `crates/cgt-core/src/matcher/acquisition_ledger.rs`: `AcquisitionLot` struct and methods
- No API changes, no serialization changes, no behavioral changes
- All existing tests should pass without modification

## Verification

All existing golden-file tests and unit tests continue to pass, confirming no behavioral change. `cargo clippy` confirms no new warnings.
