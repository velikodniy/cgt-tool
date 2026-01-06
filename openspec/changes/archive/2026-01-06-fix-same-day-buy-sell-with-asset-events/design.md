# Design: Fix Same-Day Buy+Sell with Asset Events

## Root Cause

`calculate_remaining_at_event()` in `acquisition_ledger.rs:198-256` simulates FIFO matching to determine how many shares of each lot remain at asset event time. This is used for apportioning CAPRETURN/DIVIDEND cost adjustments.

**The bug**: FIFO is wrong. UK CGT per CG51560 uses:

1. Same Day (TCGA92/S105(1)) - match against same-day acquisitions first
2. B&B (TCGA92/S106A) - match against acquisitions within 30 days after
3. S104 Pool - match against earlier acquisitions

**Example failure** (WithAssetEventsSameDay):

```
2019-11-05: BUY 20, SELL 40 (same day)
2019-11-30: CAPRETURN on 20 shares
```

Current FIFO simulation matches the SELL 40 against 2018/2019 buys first, leaving the 2019-11-05 buy unmatched. This incorrectly calculates remaining shares for the 2019-11-30 event.

Correct Same-Day rule: Match 20 of SELL against same-day BUY first, then 20 from S104 pool. After matching: 20 shares remain (from the pool, not from the same-day buy).

## Solution

Fix `calculate_remaining_at_event()` to simulate correct matching order:

```rust
fn calculate_remaining_at_event(
    &self,
    lot: &AcquisitionLot,
    event_idx: usize,
    transactions: &[GbpTransaction],
) -> Decimal {
    // ... setup unchanged ...

    for (idx, tx) in transactions.iter().enumerate() {
        if let Operation::Sell { amount, .. } = &tx.operation {
            if tx.date >= event_date {
                continue;
            }

            let mut sell_remaining = *amount;

            // 1. SAME DAY first (TCGA92/S105(1))
            for (lot_idx, lot_entry) in self.lots.iter().enumerate() {
                if sell_remaining <= Decimal::ZERO {
                    break;
                }
                if lot_entry.date == tx.date && lot_amounts[lot_idx] > Decimal::ZERO {
                    let matched = sell_remaining.min(lot_amounts[lot_idx]);
                    lot_amounts[lot_idx] -= matched;
                    sell_remaining -= matched;
                }
            }

            // 2. B&B: acquisitions within 30 days AFTER sell (TCGA92/S106A)
            for (lot_idx, lot_entry) in self.lots.iter().enumerate() {
                if sell_remaining <= Decimal::ZERO {
                    break;
                }
                let days = (lot_entry.date - tx.date).num_days();
                if days > 0 && days <= 30 && lot_amounts[lot_idx] > Decimal::ZERO {
                    let matched = sell_remaining.min(lot_amounts[lot_idx]);
                    lot_amounts[lot_idx] -= matched;
                    sell_remaining -= matched;
                }
            }

            // 3. S104: earlier acquisitions
            for (lot_idx, lot_entry) in self.lots.iter().enumerate() {
                if sell_remaining <= Decimal::ZERO {
                    break;
                }
                if lot_entry.date < tx.date && lot_amounts[lot_idx] > Decimal::ZERO {
                    let matched = sell_remaining.min(lot_amounts[lot_idx]);
                    lot_amounts[lot_idx] -= matched;
                    sell_remaining -= matched;
                }
            }
        }
    }
    // ... rest unchanged ...
}
```

## Why Not Redesign the Pipeline?

The current architecture (apply cost adjustments upfront, then match) is correct. The bug is isolated to the simulation logic in one function. A pipeline redesign would be over-engineering.

## Risks

| Risk                               | Mitigation                                                  |
| ---------------------------------- | ----------------------------------------------------------- |
| Regression in existing tests       | Run full test suite                                         |
| Performance (3 loops instead of 1) | Negligible - only runs during ledger building, not matching |
