//! Bed and Breakfast (30-day) matching rule for CGT calculations.
//!
//! Matches disposals with acquisitions within 30 days after the disposal.
//! Per TCGA92/S106A(9), B&B matching is subject to the Same Day rule in S105(1),
//! meaning Same Day matching has priority when both rules compete for the same acquisition.

use super::{MatchResult, compute_proceeds};
use crate::error::CgtError;
use crate::models::{GbpTransaction, Match, MatchRule, Operation};
use chrono::NaiveDate;
use rust_decimal::Decimal;
use std::collections::HashMap;

/// Number of days for B&B matching window.
const BNB_WINDOW_DAYS: i64 = 30;

/// Calculate the total same-day disposal quantity for a given date and ticker.
///
/// This is used to reserve shares for Same Day matching before B&B can consume them.
/// Per TCGA92/S106A(9), B&B is "subject to" Same Day rule, meaning Same Day has priority.
fn same_day_disposal_quantity(
    date: NaiveDate,
    ticker: &str,
    all_transactions: &[GbpTransaction],
) -> Decimal {
    all_transactions
        .iter()
        .filter(|tx| tx.date == date && tx.ticker == ticker)
        .filter_map(|tx| match &tx.operation {
            Operation::Sell { amount, .. } => Some(*amount),
            _ => None,
        })
        .sum()
}

fn apply_split_ratio_effect(cumulative_ratio_effect: &mut Decimal, tx: &GbpTransaction) {
    match &tx.operation {
        Operation::Split { ratio } => {
            *cumulative_ratio_effect *= *ratio;
        }
        Operation::Unsplit { ratio } => {
            if *ratio != Decimal::ZERO {
                *cumulative_ratio_effect /= *ratio;
            }
        }
        _ => {}
    }
}

fn available_for_bnb_after_reservations(
    idx: usize,
    tx: &GbpTransaction,
    buy_amount: Decimal,
    all_transactions: &[GbpTransaction],
    future_consumption: &HashMap<usize, Decimal>,
    same_day_reservations: &mut HashMap<(NaiveDate, String), Decimal>,
) -> Decimal {
    let already_reserved = future_consumption
        .get(&idx)
        .copied()
        .unwrap_or(Decimal::ZERO);

    let available_before_same_day = buy_amount - already_reserved;
    if available_before_same_day <= Decimal::ZERO {
        return Decimal::ZERO;
    }

    // Reserve shares for Same Day matching on this acquisition date.
    // Per TCGA92/S106A(9), B&B is "subject to" Same Day rule (S105(1)).
    // Reservation is tracked across all same-day lots for this date+ticker,
    // so interleaved buys cannot over-reserve.
    let reservation_key = (tx.date, tx.ticker.clone());
    let reservation_remaining = same_day_reservations
        .entry(reservation_key)
        .or_insert_with(|| same_day_disposal_quantity(tx.date, &tx.ticker, all_transactions));

    let reserve_now = available_before_same_day.min((*reservation_remaining).max(Decimal::ZERO));
    *reservation_remaining -= reserve_now;

    available_before_same_day - reserve_now
}

fn matched_buy_cost(
    matched_qty_at_buy_time: Decimal,
    buy_amount: Decimal,
    buy_price: Decimal,
    buy_fees: Decimal,
    cost_offset: Decimal,
) -> Decimal {
    let total_cost = (buy_amount * buy_price) + buy_fees + cost_offset;
    let unit_cost = if buy_amount != Decimal::ZERO {
        total_cost / buy_amount
    } else {
        Decimal::ZERO
    };

    matched_qty_at_buy_time * unit_cost
}

fn build_bnb_match(
    sell_tx: &GbpTransaction,
    acquisition_date: NaiveDate,
    matched_qty_at_sell_time: Decimal,
    sell_amount: Decimal,
    sell_price: Decimal,
    sell_fees: Decimal,
    cost: Decimal,
) -> MatchResult {
    let proceeds = compute_proceeds(matched_qty_at_sell_time, sell_amount, sell_price, sell_fees);
    let gain_or_loss = proceeds.net_proceeds - cost;

    MatchResult {
        disposal_date: sell_tx.date,
        disposal_ticker: sell_tx.ticker.clone(),
        gross_proceeds: proceeds.gross_proceeds,
        proceeds: proceeds.net_proceeds,
        match_detail: Match {
            rule: MatchRule::BedAndBreakfast,
            quantity: matched_qty_at_sell_time,
            allowable_cost: cost,
            gain_or_loss,
            acquisition_date: Some(acquisition_date),
        },
    }
}

/// Match disposal against acquisitions within 30 days (Bed & Breakfast rule).
///
/// Returns match results for any B&B acquisitions found.
///
/// This function handles splits/unsplits that occur between the sell date
/// and the B&B acquisition date by adjusting quantities accordingly.
pub fn match_bed_and_breakfast(
    sell_tx: &GbpTransaction,
    sell_idx: usize,
    remaining: &mut Decimal,
    all_transactions: &[GbpTransaction],
    cost_offsets: &[Decimal],
    future_consumption: &mut HashMap<usize, Decimal>,
    same_day_reservations: &mut HashMap<(NaiveDate, String), Decimal>,
) -> Result<Vec<MatchResult>, CgtError> {
    let mut results = Vec::new();

    let Operation::Sell {
        amount: sell_amount,
        price: sell_price,
        fees: sell_fees,
    } = &sell_tx.operation
    else {
        return Ok(results);
    };

    // Guard against division by zero (edge case: zero sell amount)
    if *sell_amount == Decimal::ZERO {
        return Ok(results);
    }

    // Track cumulative ratio effect from splits/unsplits between sell and potential buys
    let mut cumulative_ratio_effect = Decimal::ONE;

    // Find transactions after sell date, within B&B window, for same ticker
    for (idx, tx) in all_transactions.iter().enumerate().skip(sell_idx + 1) {
        if *remaining <= Decimal::ZERO {
            break;
        }

        // Must be same ticker
        if tx.ticker != sell_tx.ticker {
            continue;
        }

        let days_diff = (tx.date - sell_tx.date).num_days();

        // Must be after sell date
        if days_diff <= 0 {
            continue;
        }

        // Must be within B&B window
        if days_diff > BNB_WINDOW_DAYS {
            break;
        }

        match &tx.operation {
            Operation::Split { .. } | Operation::Unsplit { .. } => {
                apply_split_ratio_effect(&mut cumulative_ratio_effect, tx);
            }
            Operation::Buy {
                amount,
                price,
                fees,
            } => {
                let available_at_buy_time = available_for_bnb_after_reservations(
                    idx,
                    tx,
                    *amount,
                    all_transactions,
                    future_consumption,
                    same_day_reservations,
                );
                if available_at_buy_time <= Decimal::ZERO {
                    continue;
                }

                // Convert to sell-time equivalent (accounting for splits between sell and buy)
                let available_at_sell_time = available_at_buy_time / cumulative_ratio_effect;

                // Match quantity at sell time
                let matched_qty_at_sell_time = (*remaining).min(available_at_sell_time);

                // Convert back to buy-time quantity for cost calculation
                let matched_qty_at_buy_time = matched_qty_at_sell_time * cumulative_ratio_effect;

                // Get cost for the matched quantity
                let cost = matched_buy_cost(
                    matched_qty_at_buy_time,
                    *amount,
                    *price,
                    *fees,
                    cost_offsets.get(idx).copied().unwrap_or(Decimal::ZERO),
                );

                results.push(build_bnb_match(
                    sell_tx,
                    tx.date,
                    matched_qty_at_sell_time,
                    *sell_amount,
                    *sell_price,
                    *sell_fees,
                    cost,
                ));

                *remaining -= matched_qty_at_sell_time;
                let reserved_entry = future_consumption.entry(idx).or_insert(Decimal::ZERO);
                *reserved_entry += matched_qty_at_buy_time;
            }
            _ => {}
        }
    }

    Ok(results)
}
