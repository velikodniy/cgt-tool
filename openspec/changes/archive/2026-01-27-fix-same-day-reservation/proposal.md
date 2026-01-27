## Why

The B&B matching logic incorrectly allows earlier disposals to consume acquisition shares that should be reserved for Same Day matching on the acquisition date. Per TCGA92/S106A(9), B&B rules are "subject to" the Same Day rule in S105(1), meaning Same Day has absolute priority when both rules compete for the same acquisition. This causes incorrect gain/loss allocation (~£15 discrepancy in real-world testing against cgt-calc reference implementation).

## What Changes

- Modify B&B matching to reserve shares needed for Same Day matching on each acquisition date before allowing earlier disposals to consume them
- Add test case demonstrating Same Day priority over B&B from earlier disposals
- Update tax-rules documentation to clarify the interaction between Same Day and B&B rules

## Capabilities

### New Capabilities

None - this is a bug fix within existing matching functionality.

### Modified Capabilities

- `cgt-calculation`: Adding requirement for Same Day reservation priority when B&B matches compete with potential Same Day matches on the acquisition date

## Impact

- **Code**: `crates/cgt-core/src/matcher/bed_and_breakfast.rs` - must check for same-day disposals on each potential B&B acquisition date and subtract those shares from available quantity
- **Tests**: New test fixture demonstrating the scenario where B&B from earlier disposals competes with Same Day matching
- **Docs**: `docs/tax-rules.md` - clarify HMRC rule interaction per CG51560, S105(1), S106A(9)
- **Golden files**: Some existing test outputs may change if they have this edge case

## Verification

Correctness will be validated by:

1. Cross-validation against cgt-calc reference implementation using `scripts/cross-validate.py`
2. New unit test explicitly testing the Same Day reservation scenario
3. Tracing through the statutory logic: TCGA92/S105(1)(b) establishes Same Day priority; S106A(9) subordinates B&B to Same Day
