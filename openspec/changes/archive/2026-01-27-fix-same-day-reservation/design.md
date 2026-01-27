## Context

The B&B matching in `bed_and_breakfast.rs` iterates through future acquisitions within 30 days and consumes shares from the acquisition ledger. Currently, it checks `ledger.remaining_for_date(tx.date)` to get available shares, but this doesn't account for shares that will be needed for Same Day matching on that acquisition date.

Per TCGA92/S106A(9), B&B rules are "subject to" the Same Day rule in S105(1). When an acquisition date has both buys and sells, Same Day matching must be satisfied first before B&B from earlier disposals can claim remaining shares.

The reference implementation (cgt-calc) handles this by checking for same-day disposals on each potential B&B acquisition date and subtracting those shares from the available quantity.

## Goals / Non-Goals

**Goals:**

- Ensure Same Day matching has absolute priority over B&B matching per TCGA92/S106A(9)
- Pass cross-validation against cgt-calc reference implementation
- Maintain O(n) overall matching complexity
- Add clear test coverage for the edge case

**Non-Goals:**

- Changing the fundamental matching algorithm structure
- Optimizing performance beyond current O(n) complexity
- Refactoring the acquisition ledger data structure

## Decisions

### Decision 1: Calculate same-day disposal quantity inline during B&B iteration

**Choice**: When iterating through potential B&B acquisitions, calculate the total same-day disposal quantity for that date by scanning `all_transactions` and subtract it from available shares.

**Alternatives considered**:

1. **Pre-compute a map of date → same-day disposal quantity**: Would require an additional pass and data structure, adding complexity.
2. **Modify the acquisition ledger to track reservations**: Would require significant refactoring of the ledger abstraction.

**Rationale**: The inline calculation is simple, maintains the existing code structure, and the additional scan is bounded by the 30-day window (at most ~30 dates to check, each with a linear scan that's already happening).

### Decision 2: Sum all same-ticker sells on the acquisition date

**Choice**: For each potential B&B acquisition date, sum all SELL transactions for the same ticker on that date. This represents the maximum Same Day claim.

**Rationale**: Same Day matching aggregates all same-day sells (per S105(1)(a)), so the reservation must account for the total, not individual transactions.

### Decision 3: Apply reservation before split adjustment

**Choice**: Calculate the same-day disposal quantity at buy-time (after any splits), then subtract from `available_at_buy_time` before converting to sell-time equivalent.

**Rationale**: The Same Day matching happens at the acquisition date's share quantities, so the reservation should be in those units.

## Risks / Trade-offs

**[Performance] Additional scan per B&B acquisition date** → The scan is bounded by 30 days × transactions-per-day. For typical usage (few transactions per day), this is negligible. For high-frequency trading scenarios, could add overhead, but this is not a target use case.

**[Correctness] Edge case: Same Day disposal exceeds acquisition** → If same-day sells exceed same-day buys on a date, the excess goes to S104 pool per HMRC rules. The reservation should be `min(same_day_sells, acquisition_quantity)` to avoid over-reserving.

**[Golden files] Existing tests may change** → Some test fixtures may have this edge case and produce different results. This is expected and correct - the tests should be updated to match the corrected behavior.
