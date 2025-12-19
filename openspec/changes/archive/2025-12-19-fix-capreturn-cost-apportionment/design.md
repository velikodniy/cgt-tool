# Design: CAPRETURN Cost Apportionment Fix

## Context

The CGT calculator uses an acquisition ledger to track individual purchase lots before they are merged into the Section 104 pool. Corporate actions like CAPRETURN (capital return) and DIVIDEND (accumulation fund dividends) adjust the cost basis of these lots.

The cost adjustment must be apportioned across lots based on how many shares each lot contributes to the total holding at the time of the event.

## The Bug

The original formula was:

```
apportioned = adjustment × (lot_remaining / event_amount)
```

Where:

- `adjustment`: Total cost adjustment (negative for CAPRETURN, positive for DIVIDEND)
- `lot_remaining`: Shares remaining in this lot at event time
- `event_amount`: Number of shares the event applies to (from the transaction)

**Problem**: When `sum(lot_remaining for all lots) > event_amount`, each lot received more than its fair share.

**Example**:

- Hold 20 shares across 2 lots (10 each)
- CAPRETURN event: 10 shares, -£10 total
- Lot 1: -£10 × (10/10) = -£10
- Lot 2: -£10 × (10/10) = -£10
- **Total adjustment: -£20 (should be -£10)**

## The Fix

Changed formula to:

```
apportioned = adjustment × (lot_remaining / total_held)
```

Where `total_held = sum(lot_remaining for all lots at event time)`.

**Corrected Example**:

- Hold 20 shares across 2 lots (10 each)
- CAPRETURN event: 10 shares, -£10 total
- Lot 1: -£10 × (10/20) = -£5
- Lot 2: -£10 × (10/20) = -£5
- **Total adjustment: -£10 (correct)**

## Rationale

For Section 104 pooling, all shares are fungible. A corporate action affecting N shares out of M total holdings should reduce/increase the total pool cost by the stated amount, distributed proportionally across all lots based on their contribution to the pool.

The `event_amount` field indicates how many shares the event nominally applies to, but for apportionment purposes, what matters is each lot's proportion of the total holding.

## Alternatives Considered

1. **Use event_amount with clamping**: Clamp `event_amount` to `min(event_amount, total_held)`. This would work for over-apportionment but doesn't correctly handle the distribution.

2. **Track pool-level adjustments**: Apply adjustments directly to the S104 pool after shares are merged. This would require restructuring the calculation flow.

3. **Proportional by event_amount**: Only adjust lots up to `event_amount` shares worth. This is more complex and doesn't align with S104 fungibility principle.

**Decision**: Use `total_held` as denominator. Simple, correct, and aligns with how S104 pooling treats all shares as fungible.

## Risks / Trade-offs

- **Risk**: Existing calculations change

  - **Mitigation**: All test outputs regenerated; changes verified against trusted reference files

- **Risk**: Edge cases with zero holdings

  - **Mitigation**: Added guard `if total_held == Decimal::ZERO { return; }`

## Open Questions

None - the fix is straightforward and verified against reference calculations.
