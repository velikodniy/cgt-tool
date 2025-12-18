# Design: Fix report proceeds display

## Context

HMRC SA108 requires specific values for capital gains reporting:

- **Box 21 (Disposal proceeds)**: The gross amount received for the asset
- **Box 22 (Allowable costs)**: All allowable costs including purchase price, fees, and expenses

Our current implementation stores and displays net proceeds (gross - sell fees), which doesn't align with SA108 requirements. The reference test files use gross proceeds.

## Goals

- Display proceeds values that align with SA108 reporting requirements
- Maintain accuracy of gain/loss calculations (which correctly use net proceeds)
- Provide clear labeling so users understand which value to use where
- Fix display bug in same-day merge scenarios

## Non-Goals

- Loss carry-forward calculation (separate issue)
- JSON output format changes (keep as-is for backward compatibility)

## Decisions

### Decision 1: Add gross_proceeds to Disposal struct

**What**: Add a new `gross_proceeds: Decimal` field alongside existing `proceeds` (which becomes net_proceeds conceptually).

**Why**: This allows formatters to display both values without recalculating. The calculation engine already computes gross proceeds internally.

**Alternatives considered**:

- Recalculate gross from net + fees in formatter: Requires passing sell fees separately, adds complexity
- Only store gross and compute net in formatter: Breaks existing code that uses `proceeds` for calculations

### Decision 2: Summary table shows gross proceeds

**What**: The summary table's "Proceeds" column will show gross proceeds (matching SA108 Box 21).

**Why**: This is what users need to enter on their tax return. The reference files use this approach.

### Decision 3: Disposal details show both values

**What**: Each disposal breakdown will show:

```
Gross Proceeds: 10 × £4.67 = £46.70
Net Proceeds: £46.70 - £12.50 fees = £34.20
Cost: £54.07
Result: -£19.87
```

**Why**: Shows the full calculation chain so users can verify the math and understand both values.

### Decision 4: Fix weighted average price for same-day merges

**What**: When multiple sells occur on the same day, display the weighted average price instead of one arbitrary price.

**How**: Calculate `total_gross_proceeds / total_quantity` for the merged disposal.

## Risks / Trade-offs

- **Breaking change for test snapshots**: All `tests/plain/*.txt` files need updating. Mitigation: Update them as part of this change.
- **Increased output verbosity**: Reports will be slightly longer. Acceptable trade-off for clarity.

## Migration Plan

1. Update data model with new field
2. Update matchers to populate new field
3. Update formatters
4. Update all test expected outputs
5. Update documentation

Rollback: Revert the commit if issues arise.

## Open Questions

None - the approach is straightforward.
