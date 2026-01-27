//! Bed and Breakfast (30-day) matching rule for CGT calculations.
//!
//! Matches disposals with acquisitions within 30 days after the disposal.
//! Per TCGA92/S106A(9), B&B matching is subject to the Same Day rule in S105(1),
//! meaning Same Day matching has priority when both rules compete for the same acquisition.

use super::MatchResult;
use crate::error::CgtError;
use crate::models::{GbpTransaction, MatchRule, Operation};
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
            Operation::Split { ratio } => {
                cumulative_ratio_effect *= *ratio;
            }
            Operation::Unsplit { ratio } => {
                if *ratio != Decimal::ZERO {
                    cumulative_ratio_effect /= *ratio;
                }
            }
            Operation::Buy {
                amount,
                price,
                fees,
            } => {
                // Reserve shares for Same Day matching on this acquisition date.
                // Per TCGA92/S106A(9), B&B is "subject to" Same Day rule (S105(1)).
                // Cap reservation at acquisition quantity to handle edge case where
                // same-day sells exceed buys.
                let same_day_sells =
                    same_day_disposal_quantity(tx.date, &tx.ticker, all_transactions);
                let reserved_for_same_day = same_day_sells.min(*amount);
                let already_reserved = future_consumption
                    .get(&idx)
                    .copied()
                    .unwrap_or(Decimal::ZERO);
                let available_at_buy_time = *amount - reserved_for_same_day - already_reserved;
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
                let total_cost = (*amount * *price)
                    + *fees
                    + cost_offsets.get(idx).copied().unwrap_or(Decimal::ZERO);
                let unit_cost = if *amount != Decimal::ZERO {
                    total_cost / *amount
                } else {
                    Decimal::ZERO
                };
                let cost = matched_qty_at_buy_time * unit_cost;

                // Calculate proportional proceeds and fees based on sell-time quantity
                let proportion = matched_qty_at_sell_time / *sell_amount;
                let gross_proceeds = matched_qty_at_sell_time * *sell_price;
                let fees = *sell_fees * proportion;
                let net_proceeds = gross_proceeds - fees;

                let gain_or_loss = net_proceeds - cost;

                results.push(MatchResult {
                    disposal_date: sell_tx.date,
                    disposal_ticker: sell_tx.ticker.clone(),
                    quantity: matched_qty_at_sell_time,
                    gross_proceeds,
                    proceeds: net_proceeds,
                    allowable_cost: cost,
                    gain_or_loss,
                    rule: MatchRule::BedAndBreakfast,
                    acquisition_date: Some(tx.date),
                });

                *remaining -= matched_qty_at_sell_time;
                let reserved_entry = future_consumption.entry(idx).or_insert(Decimal::ZERO);
                *reserved_entry += matched_qty_at_buy_time;
            }
            _ => {}
        }
    }

    Ok(results)
}
