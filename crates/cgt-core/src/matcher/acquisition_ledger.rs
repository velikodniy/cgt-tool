//! Acquisition ledger for tracking share purchases and their costs.

use crate::models::{GbpTransaction, Operation};
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
    /// Amount moved to Section 104 pool
    pub in_pool: Decimal,
}

impl AcquisitionLot {
    /// Create a new acquisition lot.
    pub fn new(
        transaction_idx: usize,
        date: NaiveDate,
        amount: Decimal,
        price: Decimal,
        expenses: Decimal,
    ) -> Self {
        Self {
            transaction_idx,
            date,
            original_amount: amount,
            remaining_amount: amount,
            price,
            expenses,
            cost_offset: Decimal::ZERO,
            consumed: Decimal::ZERO,
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
        self.remaining_amount - self.consumed - self.in_pool
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
    ) {
        self.lots.push(AcquisitionLot::new(
            transaction_idx,
            date,
            amount,
            price,
            expenses,
        ));
    }

    /// Get total remaining shares across all lots.
    pub fn remaining_shares(&self) -> Decimal {
        self.lots.iter().map(|lot| lot.available()).sum()
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
    pub fn apply_cost_adjustment(
        &mut self,
        event_idx: usize,
        event_date: NaiveDate,
        _event_amount: Decimal,
        adjustment: Decimal,
        transactions: &[GbpTransaction],
    ) {
        // Calculate how much of each lot is left after sells before this event
        let amounts_left: Vec<Decimal> = self
            .lots
            .iter()
            .map(|lot| {
                if lot.date >= event_date {
                    return Decimal::ZERO;
                }
                self.calculate_remaining_at_event(lot, event_idx, transactions)
            })
            .collect();

        // Total shares held at the time of the event
        let total_held: Decimal = amounts_left.iter().sum();
        if total_held == Decimal::ZERO {
            return;
        }

        // Apportion the adjustment to lots based on their proportion of total holdings.
        // For S104 pooling, all shares are fungible, so the adjustment is spread
        // proportionally across all lots.
        for (i, lot) in self.lots.iter_mut().enumerate() {
            let amount_left = amounts_left[i];
            if amount_left > Decimal::ZERO && lot.date < event_date {
                let apportioned = adjustment * (amount_left / total_held);
                lot.cost_offset += apportioned;
            }
        }
    }

    /// Calculate how much of a lot remains at the time of an event.
    ///
    /// This simulates FIFO matching for sells that occurred before the event
    /// to determine how much of this lot would still be held at event time.
    fn calculate_remaining_at_event(
        &self,
        lot: &AcquisitionLot,
        event_idx: usize,
        transactions: &[GbpTransaction],
    ) -> Decimal {
        let event_date = transactions
            .get(event_idx)
            .map(|t| t.date)
            .unwrap_or(lot.date);

        let mut remaining = lot.original_amount;

        // Track amounts left in all lots (for FIFO simulation)
        let mut lot_amounts: Vec<Decimal> = self.lots.iter().map(|l| l.original_amount).collect();

        // Process sells chronologically that occurred BEFORE the event date
        for (idx, tx) in transactions.iter().enumerate() {
            if idx >= event_idx {
                break;
            }
            // Only consider sells that happened BEFORE the event date (strictly less than)
            if let Operation::Sell { amount, .. } = &tx.operation {
                if tx.date >= event_date {
                    continue;
                }

                // Match this sell against acquisitions in FIFO order
                let mut sell_remaining = *amount;
                for (lot_idx, lot_entry) in self.lots.iter().enumerate() {
                    if sell_remaining <= Decimal::ZERO {
                        break;
                    }
                    // Can only match against acquisitions that happened before this sell
                    if lot_entry.transaction_idx >= idx {
                        continue;
                    }
                    // Must be same ticker (already filtered by ledger per-ticker)
                    let available = lot_amounts[lot_idx];
                    if available > Decimal::ZERO {
                        let matched = sell_remaining.min(available);
                        lot_amounts[lot_idx] -= matched;
                        sell_remaining -= matched;
                    }
                }
            }
        }

        // Find this lot's remaining amount
        if let Some(lot_idx) = self
            .lots
            .iter()
            .position(|l| l.transaction_idx == lot.transaction_idx)
        {
            remaining = lot_amounts[lot_idx];
        }

        remaining.max(Decimal::ZERO)
    }

    /// Consume shares from lots on a specific date (for Same Day matching).
    pub fn consume_shares_on_date(&mut self, date: NaiveDate, amount: Decimal) -> Decimal {
        let mut remaining = amount;
        let mut total_cost = Decimal::ZERO;

        for lot in &mut self.lots {
            if lot.date == date && remaining > Decimal::ZERO {
                let available = lot.available();
                if available > Decimal::ZERO {
                    let to_consume = remaining.min(available);
                    total_cost += to_consume * lot.adjusted_unit_cost();
                    lot.consume(to_consume);
                    remaining -= to_consume;
                }
            }
        }

        total_cost
    }

    /// Consume shares from lots after a date (for B&B matching).
    ///
    /// Returns (consumed_amount, total_cost, acquisition_date).
    pub fn consume_shares_after_date(
        &mut self,
        sell_date: NaiveDate,
        amount: Decimal,
        max_days: i64,
    ) -> Option<(Decimal, Decimal, NaiveDate)> {
        let mut remaining = amount;

        for lot in &mut self.lots {
            let days_diff = (lot.date - sell_date).num_days();
            if days_diff > 0 && days_diff <= max_days && remaining > Decimal::ZERO {
                let available = lot.available();
                if available > Decimal::ZERO {
                    let to_consume = remaining.min(available);
                    let cost = to_consume * lot.adjusted_unit_cost();
                    lot.consume(to_consume);
                    remaining -= to_consume;

                    return Some((to_consume, cost, lot.date));
                }
            }
        }

        None
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

    /// Get a lot by transaction index.
    pub fn get_lot_by_idx(&self, transaction_idx: usize) -> Option<&AcquisitionLot> {
        self.lots
            .iter()
            .find(|lot| lot.transaction_idx == transaction_idx)
    }

    /// Get mutable lot by transaction index.
    pub fn get_lot_by_idx_mut(&mut self, transaction_idx: usize) -> Option<&mut AcquisitionLot> {
        self.lots
            .iter_mut()
            .find(|lot| lot.transaction_idx == transaction_idx)
    }

    /// Get all lots.
    pub fn lots(&self) -> &[AcquisitionLot] {
        &self.lots
    }
}
