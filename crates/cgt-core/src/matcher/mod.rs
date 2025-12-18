//! Efficient share matching for CGT calculations.
//!
//! This module provides O(n) matching algorithms using acquisition ledgers
//! instead of O(n²) nested loops.

mod acquisition_ledger;
mod bed_and_breakfast;
mod same_day;
mod section104;

pub use acquisition_ledger::{AcquisitionLedger, AcquisitionLot};

use crate::error::CgtError;
use crate::models::{GbpTransaction, MatchRule, Operation, Section104Holding};
use chrono::NaiveDate;
use rust_decimal::Decimal;
use std::collections::HashMap;

/// Result of matching a disposal against acquisitions.
#[derive(Debug, Clone)]
pub struct MatchResult {
    pub disposal_date: NaiveDate,
    pub disposal_ticker: String,
    pub quantity: Decimal,
    pub proceeds: Decimal,
    pub allowable_cost: Decimal,
    pub gain_or_loss: Decimal,
    pub rule: MatchRule,
    pub acquisition_date: Option<NaiveDate>,
}

/// Efficient matcher for CGT share matching rules.
///
/// Processes transactions in chronological order, maintaining per-ticker
/// acquisition ledgers for O(n) overall complexity.
pub struct Matcher {
    /// Per-ticker acquisition ledgers
    ledgers: HashMap<String, AcquisitionLedger>,
    /// Accumulated match results
    matches: Vec<MatchResult>,
    /// Section 104 pools (remaining after same-day and B&B)
    pools: HashMap<String, Section104Holding>,
}

impl Matcher {
    /// Create a new matcher.
    pub fn new() -> Self {
        Self {
            ledgers: HashMap::new(),
            matches: Vec::new(),
            pools: HashMap::new(),
        }
    }

    /// Process a list of GBP-converted transactions and return match results.
    ///
    /// Transactions are sorted by date and same-day transactions are merged.
    pub fn process(
        &mut self,
        transactions: Vec<GbpTransaction>,
    ) -> Result<(Vec<MatchResult>, HashMap<String, Section104Holding>), CgtError> {
        // Preprocess: sort and merge same-day transactions
        let transactions = self.preprocess(transactions);

        // Build acquisition ledgers with cost adjustments from corporate actions
        self.build_ledgers(&transactions)?;

        // Process transactions in order, grouped by date
        // This ensures same-day matching happens before shares are moved to S104 pool
        let mut i = 0;
        while i < transactions.len() {
            let current_date = transactions[i].date;

            // Find all transactions on this date
            let mut day_end = i;
            while day_end < transactions.len() && transactions[day_end].date == current_date {
                day_end += 1;
            }

            // Process all sells first (for same-day and B&B matching)
            for tx in &transactions[i..day_end] {
                if matches!(tx.operation, Operation::Sell { .. }) {
                    self.process_sell(tx, &transactions)?;
                }
            }

            // Then move remaining buys to S104 pool
            for tx in &transactions[i..day_end] {
                if matches!(tx.operation, Operation::Buy { .. }) {
                    self.move_buy_to_pool(tx)?;
                }
            }

            // Process splits/unsplits
            for tx in &transactions[i..day_end] {
                self.process_corporate_action(tx)?;
            }

            i = day_end;
        }

        Ok((
            std::mem::take(&mut self.matches),
            std::mem::take(&mut self.pools),
        ))
    }

    /// Sort transactions by date and merge same-day same-ticker buys/sells.
    fn preprocess(&self, mut transactions: Vec<GbpTransaction>) -> Vec<GbpTransaction> {
        transactions.sort_by(|a, b| a.date.cmp(&b.date));

        let mut merged = Vec::new();
        if transactions.is_empty() {
            return merged;
        }

        let mut current = transactions[0].clone();

        for next in transactions.into_iter().skip(1) {
            if next.date == current.date && next.ticker == current.ticker {
                match (&mut current.operation, next.operation) {
                    (
                        Operation::Buy {
                            amount: current_amount,
                            price: current_price,
                            fees: current_fees,
                        },
                        Operation::Buy {
                            amount: next_amount,
                            price: next_price,
                            fees: next_fees,
                        },
                    ) => {
                        // Merge using GBP values (already Decimal)
                        let total_cost =
                            (*current_amount * *current_price) + (next_amount * next_price);
                        *current_amount += next_amount;
                        if *current_amount != Decimal::ZERO {
                            *current_price = total_cost / *current_amount;
                        }
                        *current_fees += next_fees;
                    }
                    (
                        Operation::Sell {
                            amount: current_amount,
                            price: current_price,
                            fees: current_fees,
                        },
                        Operation::Sell {
                            amount: next_amount,
                            price: next_price,
                            fees: next_fees,
                        },
                    ) => {
                        // Merge using GBP values (already Decimal)
                        let total_proceeds =
                            (*current_amount * *current_price) + (next_amount * next_price);
                        *current_amount += next_amount;
                        if *current_amount != Decimal::ZERO {
                            *current_price = total_proceeds / *current_amount;
                        }
                        *current_fees += next_fees;
                    }
                    (_, next_op) => {
                        merged.push(current);
                        current = GbpTransaction {
                            date: next.date,
                            ticker: next.ticker,
                            operation: next_op,
                        };
                    }
                }
            } else {
                merged.push(current);
                current = next;
            }
        }
        merged.push(current);

        merged
    }

    /// Build acquisition ledgers from transactions.
    fn build_ledgers(&mut self, transactions: &[GbpTransaction]) -> Result<(), CgtError> {
        // First pass: create lots for all BUY transactions
        for (idx, tx) in transactions.iter().enumerate() {
            if let Operation::Buy {
                amount,
                price,
                fees,
            } = &tx.operation
            {
                let ledger = self.ledgers.entry(tx.ticker.clone()).or_default();
                ledger.add_acquisition(idx, tx.date, *amount, *price, *fees);
            }
        }

        // Second pass: apply corporate actions (CAPRETURN reduces cost, DIVIDEND increases cost)
        self.apply_corporate_actions(transactions)?;

        Ok(())
    }

    /// Apply CAPRETURN and DIVIDEND events to acquisition costs.
    fn apply_corporate_actions(&mut self, transactions: &[GbpTransaction]) -> Result<(), CgtError> {
        for (event_idx, tx) in transactions.iter().enumerate() {
            match &tx.operation {
                Operation::CapReturn {
                    amount: event_amount,
                    total_value,
                    fees: event_fees,
                } => {
                    if let Some(ledger) = self.ledgers.get_mut(&tx.ticker) {
                        let net_value = *total_value - *event_fees;
                        ledger.apply_cost_adjustment(
                            event_idx,
                            tx.date,
                            *event_amount,
                            -net_value, // Negative = reduce cost
                            transactions,
                        );
                    }
                }
                Operation::Dividend {
                    amount: event_amount,
                    total_value,
                    ..
                } => {
                    if let Some(ledger) = self.ledgers.get_mut(&tx.ticker) {
                        ledger.apply_cost_adjustment(
                            event_idx,
                            tx.date,
                            *event_amount,
                            *total_value, // Positive = increase cost
                            transactions,
                        );
                    }
                }
                _ => {}
            }
        }
        Ok(())
    }

    /// Process a sell transaction.
    fn process_sell(
        &mut self,
        tx: &GbpTransaction,
        all_transactions: &[GbpTransaction],
    ) -> Result<(), CgtError> {
        let Operation::Sell {
            amount,
            price,
            fees,
        } = &tx.operation
        else {
            return Ok(());
        };

        let mut remaining = *amount;
        let gross_proceeds = *amount * *price;
        let total_fees = *fees;

        // 1. Same Day matching
        let same_day_matched =
            same_day::match_same_day(self, tx, &mut remaining, all_transactions)?;
        for m in same_day_matched {
            self.matches.push(m);
        }

        // 2. Bed & Breakfast matching (30-day rule)
        let bnb_matched =
            bed_and_breakfast::match_bed_and_breakfast(self, tx, &mut remaining, all_transactions)?;
        for m in bnb_matched {
            self.matches.push(m);
        }

        // 3. Section 104 pool
        if remaining > Decimal::ZERO {
            let s104_matched = section104::match_section_104(
                self,
                tx,
                remaining,
                gross_proceeds,
                total_fees,
                *amount,
            )?;
            if let Some(m) = s104_matched {
                self.matches.push(m);
            }
        }

        Ok(())
    }

    /// Move remaining shares from a buy to the Section 104 pool.
    fn move_buy_to_pool(&mut self, tx: &GbpTransaction) -> Result<(), CgtError> {
        if !matches!(tx.operation, Operation::Buy { .. }) {
            return Ok(());
        }

        if let Some(ledger) = self.ledgers.get(&tx.ticker) {
            let remaining = ledger.remaining_for_date(tx.date);
            if remaining > Decimal::ZERO {
                let cost = ledger.cost_for_date(tx.date, remaining);
                let pool =
                    self.pools
                        .entry(tx.ticker.clone())
                        .or_insert_with(|| Section104Holding {
                            ticker: tx.ticker.clone(),
                            quantity: Decimal::ZERO,
                            total_cost: Decimal::ZERO,
                        });
                pool.quantity += remaining;
                pool.total_cost += cost;

                // Mark as consumed from ledger
                if let Some(ledger) = self.ledgers.get_mut(&tx.ticker) {
                    ledger.consume_for_pool(tx.date, remaining);
                }
            }
        }
        Ok(())
    }

    /// Process corporate actions (split/unsplit).
    fn process_corporate_action(&mut self, tx: &GbpTransaction) -> Result<(), CgtError> {
        match &tx.operation {
            Operation::Split { ratio } => {
                if let Some(pool) = self.pools.get_mut(&tx.ticker) {
                    pool.quantity *= *ratio;
                }
            }
            Operation::Unsplit { ratio } => {
                if let Some(pool) = self.pools.get_mut(&tx.ticker)
                    && *ratio != Decimal::ZERO
                {
                    pool.quantity /= *ratio;
                }
            }
            _ => {}
        }
        Ok(())
    }

    /// Get the ledger for a ticker.
    pub fn get_ledger(&self, ticker: &str) -> Option<&AcquisitionLedger> {
        self.ledgers.get(ticker)
    }

    /// Get mutable ledger for a ticker.
    pub fn get_ledger_mut(&mut self, ticker: &str) -> Option<&mut AcquisitionLedger> {
        self.ledgers.get_mut(ticker)
    }

    /// Get the Section 104 pool for a ticker.
    pub fn get_pool(&self, ticker: &str) -> Option<&Section104Holding> {
        self.pools.get(ticker)
    }

    /// Get mutable Section 104 pool for a ticker.
    pub fn get_pool_mut(&mut self, ticker: &str) -> Option<&mut Section104Holding> {
        self.pools.get_mut(ticker)
    }
}

impl Default for Matcher {
    fn default() -> Self {
        Self::new()
    }
}
