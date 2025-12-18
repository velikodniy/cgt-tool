//! Bed and Breakfast (30-day) matching rule for CGT calculations.
//!
//! Matches disposals with acquisitions within 30 days after the disposal.

use super::{MatchResult, Matcher};
use crate::error::CgtError;
use crate::models::{GbpTransaction, MatchRule, Operation};
use rust_decimal::Decimal;

/// Number of days for B&B matching window.
const BNB_WINDOW_DAYS: i64 = 30;

/// Match disposal against acquisitions within 30 days (Bed & Breakfast rule).
///
/// Returns match results for any B&B acquisitions found.
///
/// This function handles splits/unsplits that occur between the sell date
/// and the B&B acquisition date by adjusting quantities accordingly.
pub fn match_bed_and_breakfast(
    matcher: &mut Matcher,
    sell_tx: &GbpTransaction,
    remaining: &mut Decimal,
    all_transactions: &[GbpTransaction],
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

    // Track cumulative ratio effect from splits/unsplits between sell and potential buys
    let mut cumulative_ratio_effect = Decimal::ONE;

    // Find transactions after sell date, within B&B window, for same ticker
    for tx in all_transactions {
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
            Operation::Buy { .. } => {
                let ledger = match matcher.get_ledger_mut(&sell_tx.ticker) {
                    Some(l) => l,
                    None => continue,
                };

                // Get available shares at buy time (not yet consumed by same-day or earlier B&B)
                let available_at_buy_time = ledger.remaining_for_date(tx.date);
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
                let cost = ledger.consume_shares_on_date(tx.date, matched_qty_at_buy_time);

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
                    proceeds: net_proceeds,
                    allowable_cost: cost,
                    gain_or_loss,
                    rule: MatchRule::BedAndBreakfast,
                    acquisition_date: Some(tx.date),
                });

                *remaining -= matched_qty_at_sell_time;
            }
            _ => {}
        }
    }

    Ok(results)
}
