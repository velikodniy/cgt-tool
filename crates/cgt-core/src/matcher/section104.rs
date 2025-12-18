//! Section 104 pool matching for CGT calculations.
//!
//! Matches disposals against the Section 104 holding pool.

use super::{MatchResult, Matcher};
use crate::error::CgtError;
use crate::models::{GbpTransaction, MatchRule};
use rust_decimal::Decimal;

/// Match disposal against Section 104 pool.
///
/// Returns a match result if there are shares in the pool.
pub fn match_section_104(
    matcher: &mut Matcher,
    sell_tx: &GbpTransaction,
    remaining: Decimal,
    gross_proceeds: Decimal,
    total_fees: Decimal,
    total_sell_amount: Decimal,
) -> Result<Option<MatchResult>, CgtError> {
    if remaining == Decimal::ZERO {
        return Ok(None);
    }

    let Some(pool) = matcher.get_pool_mut(&sell_tx.ticker) else {
        return Ok(None);
    };

    if pool.quantity == Decimal::ZERO {
        return Ok(None);
    }

    // Guard against division by zero (edge case: zero sell amount)
    if total_sell_amount == Decimal::ZERO {
        return Ok(None);
    }

    // Calculate how much we can match from the pool
    let matched_qty = remaining.min(pool.quantity);

    // Calculate proportional cost from pool
    let unit_cost = if pool.quantity != Decimal::ZERO {
        pool.total_cost / pool.quantity
    } else {
        Decimal::ZERO
    };
    let cost = matched_qty * unit_cost;

    // Update pool
    pool.quantity -= matched_qty;
    pool.total_cost -= cost;

    // Calculate proportional proceeds and fees
    let proportion = matched_qty / total_sell_amount;
    let gross_portion = gross_proceeds * proportion;
    let fees = total_fees * proportion;
    let net_proceeds = gross_portion - fees;

    let gain_or_loss = net_proceeds - cost;

    Ok(Some(MatchResult {
        disposal_date: sell_tx.date,
        disposal_ticker: sell_tx.ticker.clone(),
        quantity: matched_qty,
        gross_proceeds: gross_portion,
        proceeds: net_proceeds,
        allowable_cost: cost,
        gain_or_loss,
        rule: MatchRule::Section104,
        acquisition_date: None,
    }))
}
