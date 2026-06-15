//! Phase 2 of the engine: chronological replay over the event stream and the
//! quantity-only match plan, producing priced legs.
//!
//! Per-ticker pool {quantity, cost} evolves in day order. The canonical
//! intra-day order from normalization (value events, then buys, then sells,
//! then splits) is load-bearing: corporate actions adjust the held cost basis
//! at their own date and never earlier, so a disposal's allowable cost depends
//! only on events up to and including the disposal date.

use std::collections::HashMap;

use chrono::NaiveDate;
use rust_decimal::Decimal;

use super::normalize::{Event, EventId, EventKind, EventStream, Trade};
use super::plan::{DisposalPlan, MatchPlan};
use crate::error::CgtError;

/// HMRC share matching rule for a priced leg.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum LegRule {
    SameDay,
    BedAndBreakfast,
    Section104,
}

/// A priced disposal leg.
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct PricedLeg {
    pub(crate) rule: LegRule,
    pub(crate) quantity: Decimal,
    pub(crate) allowable_cost: Decimal,
    pub(crate) gross_proceeds: Decimal,
    pub(crate) net_proceeds: Decimal,
    pub(crate) gain_or_loss: Decimal,
    pub(crate) acquisition_date: Option<NaiveDate>,
}

/// A priced disposal: all legs for one merged sell event.
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct PricedDisposal {
    pub(crate) date: NaiveDate,
    pub(crate) ticker: String,
    pub(crate) quantity: Decimal,
    pub(crate) legs: Vec<PricedLeg>,
}

/// Final pool state for a ticker after the whole replay.
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct Holding {
    pub(crate) ticker: String,
    pub(crate) quantity: Decimal,
    pub(crate) total_cost: Decimal,
}

/// Output of Phase 2.
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct ValuedReport {
    pub(crate) disposals: Vec<PricedDisposal>,
    pub(crate) holdings: Vec<Holding>,
}

#[derive(Debug, Clone, Default)]
struct Pool {
    quantity: Decimal,
    cost: Decimal,
}

/// Run the chronological replay, pricing every disposal in the plan.
pub(crate) fn value(stream: &EventStream, plan: &MatchPlan) -> Result<ValuedReport, CgtError> {
    // Disposals indexed by their sell event for O(1) lookup during the walk.
    let mut plan_by_sell: HashMap<EventId, &DisposalPlan> = HashMap::new();
    for disposal in &plan.disposals {
        plan_by_sell.insert(disposal.sell, disposal);
    }

    let mut pools: HashMap<String, Pool> = HashMap::new();
    // First-seen ticker order is irrelevant: holdings are sorted by ticker on
    // output, and every ticker that ever pooled is reported (including drained
    // pools at quantity zero).
    let mut disposals: Vec<PricedDisposal> = Vec::new();

    let events = stream.events();
    let mut day_start = 0;
    while day_start < events.len() {
        let date = events[day_start].date;
        let day_len = events[day_start..]
            .iter()
            .take_while(|event| event.date == date)
            .count();
        let day = &events[day_start..day_start + day_len];

        // The canonical order already groups value events first, then buys,
        // then sells, then splits. Walking the slice in order honours it.
        let mut day_buys: HashMap<&str, &Trade> = HashMap::new();
        for event in day {
            match &event.kind {
                EventKind::Accumulation { total_value, .. } => {
                    if let Some(pool) = pools.get_mut(&event.ticker)
                        && pool.quantity > Decimal::ZERO
                    {
                        // Held cost rises by the reinvested distribution
                        // (TCGA92 basis adjustment).
                        pool.cost += *total_value;
                    }
                }
                EventKind::CapitalReturn {
                    total_value, fees, ..
                } => {
                    if let Some(pool) = pools.get_mut(&event.ticker)
                        && pool.quantity > Decimal::ZERO
                    {
                        let net = *total_value - *fees;
                        if net > pool.cost {
                            return Err(CgtError::CapitalReturnExceedsBasis {
                                ticker: event.ticker.clone(),
                                date: event.date,
                                net,
                                basis: pool.cost,
                            });
                        }
                        // Small capital distribution reduces the pool cost
                        // (TCGA92/S122(2), CG57844).
                        pool.cost -= net;
                    }
                }
                EventKind::Buy(trade) => {
                    day_buys.insert(event.ticker.as_str(), trade);
                }
                EventKind::Sell(trade) => {
                    if let Some(plan) = plan_by_sell.get(&event.id) {
                        let priced =
                            price_disposal(event, trade, plan, stream, &mut pools, &day_buys)?;
                        disposals.push(priced);
                    }
                }
                EventKind::Dividend { .. } => {}
                EventKind::Split { ratio } => {
                    if let Some(pool) = pools.get_mut(&event.ticker) {
                        pool.quantity *= *ratio;
                    }
                }
                EventKind::Unsplit { ratio } => {
                    if let Some(pool) = pools.get_mut(&event.ticker)
                        && *ratio != Decimal::ZERO
                    {
                        pool.quantity /= *ratio;
                    }
                }
            }
        }

        // Residue of each day's buys (not consumed by same-day or reserved by
        // B&B) enters the pool after the day's sells are priced.
        for event in day {
            if let EventKind::Buy(trade) = &event.kind {
                let reserved = plan
                    .bnb_reservations
                    .get(&event.id)
                    .copied()
                    .unwrap_or(Decimal::ZERO);
                let same_day_consumed = same_day_consumed_for(event.id, &plan_by_sell, stream);
                let residue = trade.quantity - reserved - same_day_consumed;
                if residue > Decimal::ZERO {
                    // Residue cost mirrors the day buy's unit cost (path-
                    // dependent; divide-then-multiply preserved).
                    let unit = day_buy_unit_cost(trade);
                    let pool = pools.entry(event.ticker.clone()).or_default();
                    pool.quantity += residue;
                    pool.cost += residue * unit;
                }
            }
        }

        day_start += day_len;
    }

    let mut holdings: Vec<Holding> = pools
        .into_iter()
        .map(|(ticker, pool)| Holding {
            ticker,
            quantity: pool.quantity,
            total_cost: pool.cost,
        })
        .collect();
    holdings.sort_by(|a, b| a.ticker.cmp(&b.ticker));

    Ok(ValuedReport {
        disposals,
        holdings,
    })
}

/// Unit cost of a merged day buy: `(qty*price + fees)/qty`. Divide-then-
/// multiply is load-bearing for output equivalence; rust_decimal is path-
/// dependent.
fn day_buy_unit_cost(trade: &Trade) -> Decimal {
    let total = trade.quantity * trade.price + trade.fees;
    if trade.quantity != Decimal::ZERO {
        total / trade.quantity
    } else {
        Decimal::ZERO
    }
}

/// Total quantity of a buy consumed by same-day disposals on its own date.
fn same_day_consumed_for(
    buy: EventId,
    plan_by_sell: &HashMap<EventId, &DisposalPlan>,
    stream: &EventStream,
) -> Decimal {
    // A same-day buy and the same-day sells share a (date, ticker). Sum the
    // same-day legs of every disposal on that date+ticker.
    let Some(buy_event) = stream.get(buy) else {
        return Decimal::ZERO;
    };
    let mut total = Decimal::ZERO;
    for event in stream.events() {
        if event.date != buy_event.date || event.ticker != buy_event.ticker {
            continue;
        }
        if let Some(disposal) = plan_by_sell.get(&event.id)
            && let Some(leg) = &disposal.same_day
        {
            total += leg.quantity;
        }
    }
    total
}

// Pricing is filled in by a later step; this stub returns empty legs so the
// replay compiles and the pool-mutation paths can be exercised in isolation.
fn price_disposal(
    sell: &Event,
    trade: &Trade,
    _plan: &DisposalPlan,
    _stream: &EventStream,
    _pools: &mut HashMap<String, Pool>,
    _day_buys: &HashMap<&str, &Trade>,
) -> Result<PricedDisposal, CgtError> {
    Ok(PricedDisposal {
        date: sell.date,
        ticker: sell.ticker.clone(),
        quantity: trade.quantity,
        legs: Vec::new(),
    })
}

#[cfg(test)]
mod tests {
    use rust_decimal_macros::dec;

    use super::{Holding, LegRule, ValuedReport, value};
    use crate::engine::normalize::normalize;
    use crate::engine::plan::plan;
    use crate::error::CgtError;

    fn valued(input: &str) -> ValuedReport {
        let transactions = crate::dsl::parse(input).expect("test DSL parses");
        let stream = normalize(&transactions, None).expect("test input normalizes");
        let match_plan = plan(&stream).expect("test input plans");
        value(&stream, &match_plan).expect("test input values")
    }

    fn value_err(input: &str) -> CgtError {
        let transactions = crate::dsl::parse(input).expect("test DSL parses");
        let stream = normalize(&transactions, None).expect("test input normalizes");
        let match_plan = plan(&stream).expect("test input plans");
        value(&stream, &match_plan).expect_err("valuation must fail")
    }

    fn holding<'r>(report: &'r ValuedReport, ticker: &str) -> &'r Holding {
        report
            .holdings
            .iter()
            .find(|holding| holding.ticker == ticker)
            .expect("holding present")
    }

    #[test]
    fn buy_residue_grows_the_pool() {
        // A lone buy never matched by a disposal pools at its full unit cost
        // (qty*price + fees)/qty * qty = qty*price + fees.
        let report = valued("2024-01-01 BUY ABC 10 @ 10.00 GBP FEES 5.00 GBP\n");
        let pool = holding(&report, "ABC");
        assert_eq!(pool.quantity, dec!(10));
        assert_eq!(pool.total_cost, dec!(105));
    }

    #[test]
    fn accumulation_raises_pool_cost() {
        // ACCUMULATION adds the reinvested distribution to the held cost.
        let report = valued(
            "2024-01-01 BUY ABC 10 @ 10.00 GBP\n\
             2024-03-01 ACCUMULATION ABC 10 TOTAL 5.00 GBP\n",
        );
        let pool = holding(&report, "ABC");
        assert_eq!(pool.quantity, dec!(10));
        assert_eq!(pool.total_cost, dec!(105));
    }

    #[test]
    fn capital_return_lowers_pool_cost() {
        // CAPRETURN net (total - fees) reduces the held cost basis.
        let report = valued(
            "2024-01-01 BUY ABC 10 @ 10.00 GBP\n\
             2024-03-01 CAPRETURN ABC 10 TOTAL 20.00 GBP FEES 2.00 GBP\n",
        );
        let pool = holding(&report, "ABC");
        assert_eq!(pool.quantity, dec!(10));
        assert_eq!(pool.total_cost, dec!(82));
    }

    #[test]
    fn capital_return_exceeding_basis_errors() {
        // net = 150 - 0 exceeds the pool cost of 100; S122(2) does not apply.
        let err = value_err(
            "2024-01-01 BUY ABC 10 @ 10.00 GBP\n\
             2024-03-01 CAPRETURN ABC 10 TOTAL 150.00 GBP\n",
        );
        assert!(matches!(
            err,
            CgtError::CapitalReturnExceedsBasis {
                ref ticker,
                net,
                basis,
                ..
            } if ticker == "ABC" && net == dec!(150) && basis == dec!(100)
        ));
    }

    #[test]
    fn split_scales_quantity_not_cost() {
        // A SPLIT 2 doubles the pooled quantity and leaves the cost untouched.
        let report = valued(
            "2024-01-01 BUY ABC 10 @ 10.00 GBP\n\
             2024-03-01 SPLIT ABC RATIO 2\n",
        );
        let pool = holding(&report, "ABC");
        assert_eq!(pool.quantity, dec!(20));
        assert_eq!(pool.total_cost, dec!(100));
    }

    #[test]
    fn leg_rule_variants_are_distinct() {
        // The three HMRC matching rules are distinct discriminants.
        assert_ne!(LegRule::SameDay, LegRule::BedAndBreakfast);
        assert_ne!(LegRule::BedAndBreakfast, LegRule::Section104);
        assert_ne!(LegRule::SameDay, LegRule::Section104);
    }

    #[test]
    fn holdings_include_every_pooled_ticker_sorted() {
        // Every ticker that ever pooled is reported, ordered by ticker.
        let report = valued(
            "2024-01-01 BUY ZZZ 1 @ 5.00 GBP\n\
             2024-01-01 BUY ABC 2 @ 3.00 GBP\n\
             2024-01-01 BUY MMM 4 @ 1.00 GBP\n",
        );
        let tickers: Vec<&str> = report
            .holdings
            .iter()
            .map(|holding| holding.ticker.as_str())
            .collect();
        assert_eq!(tickers, ["ABC", "MMM", "ZZZ"]);
    }
}
