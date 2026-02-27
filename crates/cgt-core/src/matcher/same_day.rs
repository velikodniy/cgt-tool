//! Same Day matching rule for CGT calculations.
//!
//! Matches disposals with acquisitions on the same day.

use super::{MatchResult, Matcher};
use crate::error::CgtError;
use crate::models::{GbpTransaction, Match, MatchRule, Operation};
use rust_decimal::Decimal;

/// Match disposal against same-day acquisitions.
///
/// Returns match results for any same-day acquisitions found.
pub fn match_same_day(
    matcher: &mut Matcher,
    sell_tx: &GbpTransaction,
    remaining: &mut Decimal,
    _all_transactions: &[GbpTransaction],
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

    let Some(ledger) = matcher.get_ledger_mut(&sell_tx.ticker) else {
        return Ok(results);
    };

    // Check for same-day acquisitions
    let available = ledger.remaining_for_date(sell_tx.date);
    if available > Decimal::ZERO && *remaining > Decimal::ZERO {
        // Guard against division by zero (edge case: zero sell amount)
        if *sell_amount == Decimal::ZERO {
            return Ok(results);
        }

        let matched_qty = (*remaining).min(available);
        let cost = ledger.consume_shares_on_date(sell_tx.date, matched_qty);

        // Calculate proportional proceeds and fees (GBP values are already Decimal)
        let proportion = matched_qty / *sell_amount;
        let gross_proceeds = matched_qty * *sell_price;
        let fees = *sell_fees * proportion;
        let net_proceeds = gross_proceeds - fees;

        let gain_or_loss = net_proceeds - cost;

        results.push(MatchResult {
            disposal_date: sell_tx.date,
            disposal_ticker: sell_tx.ticker.clone(),
            gross_proceeds,
            proceeds: net_proceeds,
            match_detail: Match {
                rule: MatchRule::SameDay,
                quantity: matched_qty,
                allowable_cost: cost,
                gain_or_loss,
                acquisition_date: Some(sell_tx.date),
            },
        });

        *remaining -= matched_qty;
    }

    Ok(results)
}
