//! Section 104 pool matching for CGT calculations.
//!
//! Matches disposals against the Section 104 holding pool.

use super::{MatchResult, Matcher, compute_proceeds};
use crate::error::CgtError;
use crate::models::{GbpTransaction, Match, MatchRule, Operation};
use rust_decimal::Decimal;

/// Match disposal against Section 104 pool.
///
/// Returns a match result if there are shares in the pool.
pub fn match_section_104(
    matcher: &mut Matcher,
    sell_tx: &GbpTransaction,
    remaining: &mut Decimal,
    total_sell_amount: Decimal,
) -> Result<Option<MatchResult>, CgtError> {
    if *remaining == Decimal::ZERO {
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
    let matched_qty = (*remaining).min(pool.quantity);
    if matched_qty == Decimal::ZERO {
        return Ok(None);
    }

    let Operation::Sell {
        price: sell_price,
        fees: sell_fees,
        ..
    } = &sell_tx.operation
    else {
        return Ok(None);
    };

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
    *remaining -= matched_qty;

    let proceeds = compute_proceeds(matched_qty, total_sell_amount, *sell_price, *sell_fees);

    let gain_or_loss = proceeds.net_proceeds - cost;

    Ok(Some(MatchResult {
        disposal_date: sell_tx.date,
        disposal_ticker: sell_tx.ticker.clone(),
        gross_proceeds: proceeds.gross_proceeds,
        proceeds: proceeds.net_proceeds,
        match_detail: Match {
            rule: MatchRule::Section104,
            quantity: matched_qty,
            allowable_cost: cost,
            gain_or_loss,
            acquisition_date: None,
        },
    }))
}
