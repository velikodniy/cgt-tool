//! Bed and Breakfast (30-day) matching rule for CGT calculations.
//!
//! Matches disposals with acquisitions within 30 days after the disposal.

use super::{MatchResult, Matcher};
use crate::error::CgtError;
use crate::models::{MatchRule, Operation, Transaction};
use rust_decimal::Decimal;

/// Number of days for B&B matching window.
const BNB_WINDOW_DAYS: i64 = 30;

/// Match disposal against acquisitions within 30 days (Bed & Breakfast rule).
///
/// Returns match results for any B&B acquisitions found.
pub fn match_bed_and_breakfast(
    matcher: &mut Matcher,
    sell_tx: &Transaction,
    remaining: &mut Decimal,
    _all_transactions: &[Transaction],
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

    // Keep matching until we've matched all remaining shares or exhausted B&B opportunities
    while *remaining > Decimal::ZERO {
        let ledger = match matcher.get_ledger_mut(&sell_tx.ticker) {
            Some(l) => l,
            None => break,
        };

        let bnb_result =
            ledger.consume_shares_after_date(sell_tx.date, *remaining, BNB_WINDOW_DAYS);

        match bnb_result {
            Some((matched_qty, cost, acquisition_date)) => {
                // Calculate proportional proceeds and fees (using GBP values)
                let proportion = matched_qty / *sell_amount;
                let proceeds = matched_qty * sell_price.gbp;
                let fees = sell_fees.gbp * proportion;

                let gain_or_loss = proceeds - cost - fees;

                results.push(MatchResult {
                    disposal_date: sell_tx.date,
                    disposal_ticker: sell_tx.ticker.clone(),
                    quantity: matched_qty,
                    proceeds,
                    allowable_cost: cost + fees,
                    gain_or_loss,
                    rule: MatchRule::BedAndBreakfast,
                    acquisition_date: Some(acquisition_date),
                });

                *remaining -= matched_qty;
            }
            None => break,
        }
    }

    Ok(results)
}
