## Context

`AcquisitionLot` has a `remaining_amount` field that is initialized to `original_amount` in the constructor and never mutated. Two methods (`available()` and `held_for_adjustment()`) read it instead of `original_amount`, creating the false impression that these values could diverge.

## Goals / Non-Goals

**Goals:**

- Remove the dead `remaining_amount` field
- Replace all reads of `remaining_amount` with `original_amount`
- Confirm zero behavioral change via existing test suite

**Non-Goals:**

- Refactoring other parts of `AcquisitionLot` or the matcher
- Changing any public API or serialization format

## Decisions

**Direct replacement**: Replace `remaining_amount` with `original_amount` in `available()` and `held_for_adjustment()`, then remove the field. This is a mechanical substitution — no alternative designs needed since the values are provably identical.

## Risks / Trade-offs

- [Risk: Hidden external usage] → Mitigated by codebase-wide grep confirming `remaining_amount` appears only in `acquisition_ledger.rs` (4 occurrences, all within the struct definition and methods).
- [Risk: Future need for remaining tracking] → If needed later, it can be reintroduced with proper mutation semantics. Dead state is worse than absent state.
