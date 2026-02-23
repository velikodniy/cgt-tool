//! Acquisition ledger for tracking share purchases and their costs.

use chrono::NaiveDate;
use rust_decimal::Decimal;

/// An individual acquisition lot.
#[derive(Debug, Clone)]
pub struct AcquisitionLot {
    /// Index in the original transaction list
    pub transaction_idx: usize,
    /// Date of acquisition
    pub date: NaiveDate,
    /// Original quantity purchased
    pub original_amount: Decimal,
    /// Remaining quantity (not yet matched)
    pub remaining_amount: Decimal,
    /// Price per share
    pub price: Decimal,
    /// Transaction expenses
    pub expenses: Decimal,
    /// Cost adjustment from CAPRETURN/DIVIDEND events
    pub cost_offset: Decimal,
    /// Amount consumed by Same Day / B&B matching
    pub consumed: Decimal,
    /// Amount reserved for future B&B matches
    pub reserved: Decimal,
    /// Amount moved to Section 104 pool
    pub in_pool: Decimal,
}

#[derive(Debug, Clone, Copy)]
pub struct AcquisitionExtras {
    pub cost_offset: Decimal,
    pub reserved: Decimal,
}

impl AcquisitionExtras {
    pub fn new(cost_offset: Decimal, reserved: Decimal) -> Self {
        Self {
            cost_offset,
            reserved,
        }
    }
}

impl AcquisitionLot {
    /// Create a new acquisition lot.
    pub fn new(
        transaction_idx: usize,
        date: NaiveDate,
        amount: Decimal,
        price: Decimal,
        expenses: Decimal,
        cost_offset: Decimal,
        reserved: Decimal,
    ) -> Self {
        Self {
            transaction_idx,
            date,
            original_amount: amount,
            remaining_amount: amount,
            price,
            expenses,
            cost_offset,
            consumed: Decimal::ZERO,
            reserved,
            in_pool: Decimal::ZERO,
        }
    }

    /// Calculate base cost (before adjustments).
    pub fn base_cost(&self) -> Decimal {
        (self.original_amount * self.price) + self.expenses
    }

    /// Calculate adjusted total cost.
    pub fn adjusted_cost(&self) -> Decimal {
        self.base_cost() + self.cost_offset
    }

    /// Calculate adjusted cost per share.
    pub fn adjusted_unit_cost(&self) -> Decimal {
        if self.original_amount != Decimal::ZERO {
            self.adjusted_cost() / self.original_amount
        } else {
            Decimal::ZERO
        }
    }

    /// Get available amount (not consumed by matching).
    pub fn available(&self) -> Decimal {
        self.remaining_amount - self.consumed - self.reserved - self.in_pool
    }

    /// Get the amount held for corporate action adjustments.
    pub fn held_for_adjustment(&self) -> Decimal {
        self.remaining_amount - self.consumed
    }

    /// Consume shares for matching (Same Day or B&B).
    pub fn consume(&mut self, amount: Decimal) {
        self.consumed += amount;
    }

    /// Move shares to Section 104 pool.
    pub fn move_to_pool(&mut self, amount: Decimal) {
        self.in_pool += amount;
    }
}

/// Ledger tracking all acquisitions for a single ticker.
#[derive(Debug, Clone, Default)]
pub struct AcquisitionLedger {
    /// All acquisition lots, in chronological order
    lots: Vec<AcquisitionLot>,
}

impl AcquisitionLedger {
    /// Create a new empty ledger.
    pub fn new() -> Self {
        Self { lots: Vec::new() }
    }

    /// Add an acquisition to the ledger.
    pub fn add_acquisition(
        &mut self,
        transaction_idx: usize,
        date: NaiveDate,
        amount: Decimal,
        price: Decimal,
        expenses: Decimal,
        extras: AcquisitionExtras,
    ) {
        self.lots.push(AcquisitionLot::new(
            transaction_idx,
            date,
            amount,
            price,
            expenses,
            extras.cost_offset,
            extras.reserved,
        ));
    }

    /// Get remaining shares for a specific acquisition date.
    pub fn remaining_for_date(&self, date: NaiveDate) -> Decimal {
        self.lots
            .iter()
            .filter(|lot| lot.date == date)
            .map(|lot| lot.available())
            .sum()
    }

    /// Get the cost for shares from a specific date.
    pub fn cost_for_date(&self, date: NaiveDate, amount: Decimal) -> Decimal {
        let mut remaining = amount;
        let mut total_cost = Decimal::ZERO;

        for lot in &self.lots {
            if lot.date == date && remaining > Decimal::ZERO {
                let available = lot.available();
                if available > Decimal::ZERO {
                    let to_use = remaining.min(available);
                    total_cost += to_use * lot.adjusted_unit_cost();
                    remaining -= to_use;
                }
            }
        }

        total_cost
    }

    /// Apply a cost adjustment (from CAPRETURN or DIVIDEND) to acquisition lots.
    ///
    /// The adjustment is apportioned based on remaining shares at the time of the event.
    /// For S104 pooling, all shares are fungible, so we apportion based on each lot's
    /// proportion of total holdings, not based on the event amount.
    ///
    /// Returns any unapplied adjustment:
    /// - `Decimal::ZERO` when fully applied
    /// - negative value when a CAPRETURN reduction exceeds available basis
    /// - positive value when no holdings exist to apply an increase
    pub fn apply_cost_adjustment(&mut self, adjustment: Decimal) -> Decimal {
        if adjustment == Decimal::ZERO {
            return Decimal::ZERO;
        }

        let mut weighted_lots: Vec<(usize, Decimal)> = self
            .lots
            .iter()
            .enumerate()
            .filter_map(|(idx, lot)| {
                let held = lot.held_for_adjustment();
                (held > Decimal::ZERO).then_some((idx, held))
            })
            .collect();

        if weighted_lots.is_empty() {
            return adjustment;
        }

        if adjustment > Decimal::ZERO {
            let total_held: Decimal = weighted_lots.iter().map(|(_, held)| *held).sum();
            if total_held == Decimal::ZERO {
                return adjustment;
            }

            let mut remaining = adjustment;
            for (position, (idx, held)) in weighted_lots.iter().enumerate() {
                let apportioned = if position + 1 == weighted_lots.len() {
                    remaining
                } else {
                    adjustment * (*held / total_held)
                };

                if let Some(lot) = self.lots.get_mut(*idx) {
                    lot.cost_offset += apportioned;
                }
                remaining -= apportioned;
            }

            return Decimal::ZERO;
        }

        // Negative adjustment (CAPRETURN): cap reductions so adjusted lot cost never goes below zero.
        // Any unapplied reduction becomes a deemed gain.
        let mut remaining_reduction = -adjustment;

        loop {
            weighted_lots.retain(|(idx, held)| {
                if *held <= Decimal::ZERO {
                    return false;
                }
                self.lots
                    .get(*idx)
                    .map(|lot| lot.adjusted_cost() > Decimal::ZERO)
                    .unwrap_or(false)
            });

            if remaining_reduction <= Decimal::ZERO || weighted_lots.is_empty() {
                break;
            }

            let total_held: Decimal = weighted_lots.iter().map(|(_, held)| *held).sum();
            if total_held == Decimal::ZERO {
                break;
            }

            let mut pass_remaining = remaining_reduction;
            for (position, (idx, held)) in weighted_lots.iter().enumerate() {
                let capacity = self
                    .lots
                    .get(*idx)
                    .map(|lot| lot.adjusted_cost().max(Decimal::ZERO))
                    .unwrap_or(Decimal::ZERO);

                if capacity <= Decimal::ZERO {
                    continue;
                }

                let requested = if position + 1 == weighted_lots.len() {
                    pass_remaining
                } else {
                    remaining_reduction * (*held / total_held)
                };

                let applied = requested.min(capacity);
                if applied > Decimal::ZERO {
                    if let Some(lot) = self.lots.get_mut(*idx) {
                        lot.cost_offset -= applied;
                    }
                    pass_remaining -= applied;
                }
            }

            if pass_remaining >= remaining_reduction {
                break;
            }

            remaining_reduction = pass_remaining;
        }

        if remaining_reduction > Decimal::ZERO {
            -remaining_reduction
        } else {
            Decimal::ZERO
        }
    }

    /// Consume shares from lots on a specific date (for Same Day matching).
    pub fn consume_shares_on_date(&mut self, date: NaiveDate, amount: Decimal) -> Decimal {
        let mut total_available = Decimal::ZERO;
        let mut total_cost = Decimal::ZERO;
        let mut lots_on_date = Vec::new();

        for (idx, lot) in self.lots.iter().enumerate() {
            if lot.date == date {
                let available = lot.available();
                if available > Decimal::ZERO {
                    total_available += available;
                    total_cost += available * lot.adjusted_unit_cost();
                    lots_on_date.push((idx, available));
                }
            }
        }

        if total_available == Decimal::ZERO || amount <= Decimal::ZERO {
            return Decimal::ZERO;
        }

        let matched = amount.min(total_available);
        let ratio = matched / total_available;
        let mut remaining = matched;

        for (pos, (idx, available)) in lots_on_date.iter().enumerate() {
            let to_consume = if pos + 1 == lots_on_date.len() {
                remaining.min(*available)
            } else {
                let proportional = *available * ratio;
                if proportional > remaining {
                    remaining
                } else {
                    proportional
                }
            };

            if to_consume > Decimal::ZERO {
                if let Some(lot) = self.lots.get_mut(*idx) {
                    lot.consume(to_consume);
                }
                remaining -= to_consume;
            }
        }

        let average_cost = total_cost / total_available;
        matched * average_cost
    }

    /// Consume shares from lots before a date (for cost-basis tracking).
    pub fn consume_shares_before_date(&mut self, date: NaiveDate, amount: Decimal) {
        let mut remaining = amount;

        for lot in &mut self.lots {
            if lot.date < date && remaining > Decimal::ZERO {
                let available = lot.available();
                if available > Decimal::ZERO {
                    let to_consume = remaining.min(available);
                    lot.consume(to_consume);
                    remaining -= to_consume;
                }
            }
        }
    }

    /// Move shares to Section 104 pool for a specific date.
    pub fn consume_for_pool(&mut self, date: NaiveDate, amount: Decimal) {
        let mut remaining = amount;

        for lot in &mut self.lots {
            if lot.date == date && remaining > Decimal::ZERO {
                let available = lot.available();
                if available > Decimal::ZERO {
                    let to_move = remaining.min(available);
                    lot.move_to_pool(to_move);
                    remaining -= to_move;
                }
            }
        }
    }

    /// Get all lots.
    pub fn lots(&self) -> &[AcquisitionLot] {
        &self.lots
    }
}
