//! Efficient share matching for CGT calculations.
//!
//! This module provides O(n) matching algorithms using acquisition ledgers
//! instead of O(n²) nested loops.

mod acquisition_ledger;
mod bed_and_breakfast;
mod same_day;
mod section104;

pub use acquisition_ledger::{AcquisitionExtras, AcquisitionLedger, AcquisitionLot};

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
    /// Gross proceeds before sale fees (quantity × unit price).
    pub gross_proceeds: Decimal,
    /// Net proceeds after sale fees (gross_proceeds - fees).
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

        let cost_offsets = self.compute_cost_offsets(&transactions);
        let mut future_consumption: HashMap<usize, Decimal> = HashMap::new();

        // Process transactions in order, grouped by date
        // Buys are added before same-day sells for matching; cost offsets already applied.
        let mut i = 0;
        while i < transactions.len() {
            let current_date = transactions[i].date;

            // Find all transactions on this date
            let mut day_end = i;
            while day_end < transactions.len() && transactions[day_end].date == current_date {
                day_end += 1;
            }

            // Add buys for the day (apply cost offsets and future reservations)
            for (offset, tx) in transactions[i..day_end].iter().enumerate() {
                if let Operation::Buy {
                    amount,
                    price,
                    fees,
                } = &tx.operation
                {
                    let idx = i + offset;
                    let reserved = future_consumption.remove(&idx).unwrap_or(Decimal::ZERO);
                    if reserved > *amount {
                        return Err(CgtError::InvalidTransaction(format!(
                            "B&B reservation exceeds buy amount for {} on {}",
                            tx.ticker, tx.date
                        )));
                    }
                    let cost_offset = cost_offsets.get(idx).copied().unwrap_or(Decimal::ZERO);
                    let ledger = self.ledgers.entry(tx.ticker.clone()).or_default();
                    ledger.add_acquisition(
                        idx,
                        tx.date,
                        *amount,
                        *price,
                        *fees,
                        AcquisitionExtras::new(cost_offset, reserved),
                    );
                }
            }

            // Process all sells (same-day, B&B, then S104)
            for (offset, tx) in transactions[i..day_end].iter().enumerate() {
                if matches!(tx.operation, Operation::Sell { .. }) {
                    let idx = i + offset;
                    self.process_sell(
                        tx,
                        idx,
                        &transactions,
                        &cost_offsets,
                        &mut future_consumption,
                    )?;
                }
            }

            // Move remaining buys to S104 pool
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

    fn compute_cost_offsets(&self, transactions: &[GbpTransaction]) -> Vec<Decimal> {
        let mut ledgers: HashMap<String, AcquisitionLedger> = HashMap::new();
        let mut i = 0;

        while i < transactions.len() {
            let current_date = transactions[i].date;
            let mut day_end = i;
            while day_end < transactions.len() && transactions[day_end].date == current_date {
                day_end += 1;
            }

            // Apply corporate actions for the day (before same-day buys)
            for tx in &transactions[i..day_end] {
                match &tx.operation {
                    Operation::CapReturn {
                        total_value, fees, ..
                    } => {
                        if let Some(ledger) = ledgers.get_mut(&tx.ticker) {
                            let net_value = *total_value - *fees;
                            ledger.apply_cost_adjustment(-net_value);
                        }
                    }
                    Operation::Dividend { total_value, .. } => {
                        if let Some(ledger) = ledgers.get_mut(&tx.ticker) {
                            ledger.apply_cost_adjustment(*total_value);
                        }
                    }
                    _ => {}
                }
            }

            // Add buys for the day
            for (offset, tx) in transactions[i..day_end].iter().enumerate() {
                if let Operation::Buy {
                    amount,
                    price,
                    fees,
                } = &tx.operation
                {
                    let idx = i + offset;
                    let ledger = ledgers.entry(tx.ticker.clone()).or_default();
                    ledger.add_acquisition(
                        idx,
                        tx.date,
                        *amount,
                        *price,
                        *fees,
                        AcquisitionExtras::new(Decimal::ZERO, Decimal::ZERO),
                    );
                }
            }

            // Process sells for the day using Same Day then S104 (no B&B)
            for tx in &transactions[i..day_end] {
                if let Operation::Sell { amount, .. } = &tx.operation
                    && let Some(ledger) = ledgers.get_mut(&tx.ticker)
                {
                    let available_same_day = ledger.remaining_for_date(tx.date);
                    if available_same_day > Decimal::ZERO {
                        let matched = (*amount).min(available_same_day);
                        ledger.consume_shares_on_date(tx.date, matched);
                        let remaining = *amount - matched;
                        if remaining > Decimal::ZERO {
                            ledger.consume_shares_before_date(tx.date, remaining);
                        }
                    } else {
                        ledger.consume_shares_before_date(tx.date, *amount);
                    }
                }
            }

            i = day_end;
        }

        let mut offsets = vec![Decimal::ZERO; transactions.len()];
        for ledger in ledgers.values() {
            for lot in ledger.lots() {
                offsets[lot.transaction_idx] = lot.cost_offset;
            }
        }

        offsets
    }

    /// Process a sell transaction.
    fn process_sell(
        &mut self,
        tx: &GbpTransaction,
        sell_idx: usize,
        all_transactions: &[GbpTransaction],
        cost_offsets: &[Decimal],
        future_consumption: &mut HashMap<usize, Decimal>,
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
        let bnb_matched = bed_and_breakfast::match_bed_and_breakfast(
            tx,
            sell_idx,
            &mut remaining,
            all_transactions,
            cost_offsets,
            future_consumption,
        )?;
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
            Operation::Buy { .. }
            | Operation::Sell { .. }
            | Operation::Dividend { .. }
            | Operation::CapReturn { .. } => {}
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
