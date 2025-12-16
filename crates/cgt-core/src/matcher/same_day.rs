//! Same Day matching rule for CGT calculations.
//!
//! Matches disposals with acquisitions on the same day.

use super::{MatchResult, Matcher};
use crate::error::CgtError;
use crate::models::{MatchRule, Operation, Transaction};
use rust_decimal::Decimal;

/// Match disposal against same-day acquisitions.
///
/// Returns match results for any same-day acquisitions found.
pub fn match_same_day(
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

    let ledger = match matcher.get_ledger_mut(&sell_tx.ticker) {
        Some(l) => l,
        None => return Ok(results),
    };

    // Check for same-day acquisitions
    let available = ledger.remaining_for_date(sell_tx.date);
    if available > Decimal::ZERO && *remaining > Decimal::ZERO {
        let matched_qty = (*remaining).min(available);
        let cost = ledger.consume_shares_on_date(sell_tx.date, matched_qty);

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
            rule: MatchRule::SameDay,
            acquisition_date: Some(sell_tx.date),
        });

        *remaining -= matched_qty;
    }

    Ok(results)
}
