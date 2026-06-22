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

    for day in stream.events().chunk_by(|a, b| a.date == b.date) {
        // The canonical order already groups value events first, then buys,
        // then sells, then splits. Walking the slice in order honours it.
        let mut day_buys: HashMap<&str, &Trade> = HashMap::new();
        for event in day {
            match &event.kind {
                EventKind::Accumulation { total_value, .. } => {
                    // Reinvested distribution raises held cost (TCGA92).
                    match pools.get_mut(&event.ticker) {
                        Some(pool) if pool.quantity > Decimal::ZERO => {
                            pool.cost += *total_value;
                        }
                        // Crediting a non-held position: error, don't drop it.
                        _ if *total_value > Decimal::ZERO => {
                            return Err(CgtError::AccumulationWithoutHolding {
                                ticker: event.ticker.clone(),
                                date: event.date,
                            });
                        }
                        _ => {}
                    }
                }
                EventKind::CapitalReturn {
                    total_value, fees, ..
                } => {
                    // Exceeds-basis must error at any quantity, even against a
                    // drained or never-held holding (CG57847); read basis via
                    // get so a never-held ticker errors without a phantom pool.
                    let net = *total_value - *fees;
                    let basis = pools
                        .get(&event.ticker)
                        .map_or(Decimal::ZERO, |pool| pool.cost);
                    if net > basis {
                        return Err(CgtError::CapitalReturnExceedsBasis {
                            ticker: event.ticker.clone(),
                            date: event.date,
                            net,
                            basis,
                        });
                    }
                    if let Some(pool) = pools.get_mut(&event.ticker) {
                        pool.cost -= net; // TCGA92/S122(2)
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

        // Quantity each ticker's same-day disposals consume on this day, summed
        // in stream order from the day's disposal legs. Computed once per day so
        // the residue loop below is a lookup rather than a full-stream rescan.
        let mut same_day_consumed: HashMap<&str, Decimal> = HashMap::new();
        for event in day {
            if let Some(disposal) = plan_by_sell.get(&event.id)
                && let Some(leg) = &disposal.same_day
            {
                *same_day_consumed.entry(event.ticker.as_str()).or_default() += leg.quantity;
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
                let consumed = same_day_consumed
                    .get(event.ticker.as_str())
                    .copied()
                    .unwrap_or(Decimal::ZERO);
                let residue = trade.quantity - reserved - consumed;
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

/// Proportional disposal proceeds for a matched quantity. The fee proportion
/// `matched/sell_qty` and the order of operations are load-bearing for output
/// equivalence; rust_decimal is path-dependent.
fn proceeds(
    matched: Decimal,
    sell_qty: Decimal,
    sell_price: Decimal,
    sell_fees: Decimal,
) -> (Decimal, Decimal) {
    if sell_qty == Decimal::ZERO {
        return (Decimal::ZERO, Decimal::ZERO);
    }
    let proportion = matched / sell_qty;
    let gross = matched * sell_price;
    let fees = sell_fees * proportion;
    (gross, gross - fees)
}

/// Price every leg of one disposal. The plan's leg quantities drive pricing;
/// value never re-derives matching. Same-day and B&B legs leave the pool
/// untouched; only the Section 104 leg drains it.
fn price_disposal(
    sell: &Event,
    trade: &Trade,
    plan: &DisposalPlan,
    stream: &EventStream,
    pools: &mut HashMap<String, Pool>,
    day_buys: &HashMap<&str, &Trade>,
) -> Result<PricedDisposal, CgtError> {
    let sell_qty = trade.quantity;
    let sell_price = trade.price;
    let sell_fees = trade.fees;
    let mut legs: Vec<PricedLeg> = Vec::new();

    // 1. Same Day (TCGA92/S105(1)): cost off the merged same-day buy's unit
    //    cost. matched * (day_buy_total/day_buy_qty); divide-then-multiply.
    if let Some(leg) = &plan.same_day {
        let buy =
            day_buys
                .get(sell.ticker.as_str())
                .ok_or_else(|| CgtError::NoPriorAcquisitions {
                    ticker: sell.ticker.clone(),
                    date: sell.date,
                    attempted: sell_qty,
                })?;
        let unit = day_buy_unit_cost(buy);
        let cost = leg.quantity * unit;
        let (gross, net) = proceeds(leg.quantity, sell_qty, sell_price, sell_fees);
        legs.push(PricedLeg {
            rule: LegRule::SameDay,
            quantity: leg.quantity,
            allowable_cost: cost,
            gross_proceeds: gross,
            net_proceeds: net,
            gain_or_loss: net - cost,
            acquisition_date: Some(sell.date),
        });
    }

    // 2. Bed & Breakfast (TCGA92/S106A): price off the raw merged target buy.
    //    A B&B-consumed share is never held at a later corporate action, so no
    //    cost offset applies here.
    for bnb in &plan.bed_and_breakfast {
        let buy_event = stream
            .get(bnb.buy)
            .ok_or_else(|| CgtError::NoPriorAcquisitions {
                ticker: sell.ticker.clone(),
                date: sell.date,
                attempted: sell_qty,
            })?;
        let EventKind::Buy(buy) = &buy_event.kind else {
            return Err(CgtError::NoPriorAcquisitions {
                ticker: sell.ticker.clone(),
                date: sell.date,
                attempted: sell_qty,
            });
        };
        let unit = day_buy_unit_cost(buy);
        let cost = bnb.quantity_at_buy_scale * unit;
        let (gross, net) = proceeds(bnb.quantity_at_sell_scale, sell_qty, sell_price, sell_fees);
        legs.push(PricedLeg {
            rule: LegRule::BedAndBreakfast,
            quantity: bnb.quantity_at_sell_scale,
            allowable_cost: cost,
            gross_proceeds: gross,
            net_proceeds: net,
            gain_or_loss: net - cost,
            acquisition_date: Some(bnb.acquisition_date),
        });
    }

    // 3. Section 104: cost off the pool average at this moment.
    //    matched * (pool_cost/pool_qty); divide-then-multiply, then drain.
    if let Some(leg) = &plan.section_104 {
        let pool =
            pools
                .get_mut(sell.ticker.as_str())
                .ok_or_else(|| CgtError::NoPriorAcquisitions {
                    ticker: sell.ticker.clone(),
                    date: sell.date,
                    attempted: sell_qty,
                })?;
        let unit = if pool.quantity != Decimal::ZERO {
            pool.cost / pool.quantity
        } else {
            Decimal::ZERO
        };
        let cost = leg.quantity * unit;
        pool.quantity -= leg.quantity;
        pool.cost -= cost;
        let (gross, net) = proceeds(leg.quantity, sell_qty, sell_price, sell_fees);
        legs.push(PricedLeg {
            rule: LegRule::Section104,
            quantity: leg.quantity,
            allowable_cost: cost,
            gross_proceeds: gross,
            net_proceeds: net,
            gain_or_loss: net - cost,
            acquisition_date: None,
        });
    }

    Ok(PricedDisposal {
        date: sell.date,
        ticker: sell.ticker.clone(),
        quantity: sell_qty,
        legs,
    })
}

#[cfg(test)]
mod tests {
    use rust_decimal_macros::dec;

    use chrono::NaiveDate;

    use super::{Holding, LegRule, PricedLeg, ValuedReport, value};
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

    fn leg<'r>(report: &'r ValuedReport, date: &str, rule: LegRule) -> &'r PricedLeg {
        let on = NaiveDate::parse_from_str(date, "%Y-%m-%d").expect("test date parses");
        report
            .disposals
            .iter()
            .filter(|disposal| disposal.date == on)
            .flat_map(|disposal| disposal.legs.iter())
            .find(|leg| leg.rule == rule)
            .expect("priced leg present")
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
    fn capital_return_after_full_disposal_errors() {
        // Pool drained to zero basis: the return is no longer silently dropped.
        let err = value_err(
            "2024-01-01 BUY ABC 10 @ 10.00 GBP\n\
             2024-02-01 SELL ABC 10 @ 12.00 GBP\n\
             2024-03-01 CAPRETURN ABC 10 TOTAL 20.00 GBP\n",
        );
        assert!(matches!(
            err,
            CgtError::CapitalReturnExceedsBasis { basis, .. } if basis == dec!(0)
        ));
    }

    #[test]
    fn accumulation_without_holding_errors() {
        let err = value_err("2024-01-01 ACCUMULATION ABC 5 TOTAL 10.00 GBP\n");
        assert!(matches!(
            err,
            CgtError::AccumulationWithoutHolding { ref ticker, .. } if ticker == "ABC"
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

    #[test]
    fn section_104_leg_prices_off_the_pool_average_and_drains_it() {
        // Pool unit cost (10*10 + 5)/10 = 10.5; 4 shares cost 4*10.5 = 42.
        // Drain leaves 6 shares at 105 - 42 = 63.
        let report = valued(
            "2024-01-01 BUY ABC 10 @ 10.00 GBP FEES 5.00 GBP\n\
             2024-06-01 SELL ABC 4 @ 20.00 GBP FEES 2.00 GBP\n",
        );

        let s104 = leg(&report, "2024-06-01", LegRule::Section104);
        assert_eq!(s104.allowable_cost, dec!(42));
        assert_eq!(s104.gross_proceeds, dec!(80));
        assert_eq!(s104.net_proceeds, dec!(78));
        assert_eq!(s104.gain_or_loss, dec!(36));

        let pool = holding(&report, "ABC");
        assert_eq!(pool.quantity, dec!(6));
        assert_eq!(pool.total_cost, dec!(63));
    }

    #[test]
    fn same_day_and_section_104_legs_price_independently() {
        // The 2024-06-01 buy (5 @ 12 = 60) is same-day with the sell; it prices
        // the 5-share same-day leg at 5*(60/5) = 60 and never touches the pool.
        // The remaining 3 shares match the 2024-01-01 pool at 100/10 = 10.
        let report = valued(
            "2024-01-01 BUY ABC 10 @ 10.00 GBP\n\
             2024-06-01 BUY ABC 5 @ 12.00 GBP\n\
             2024-06-01 SELL ABC 8 @ 13.00 GBP\n",
        );

        let same_day = leg(&report, "2024-06-01", LegRule::SameDay);
        assert_eq!(same_day.allowable_cost, dec!(60));
        assert_eq!(same_day.gross_proceeds, dec!(65));
        assert_eq!(same_day.net_proceeds, dec!(65));
        assert_eq!(same_day.gain_or_loss, dec!(5));

        let s104 = leg(&report, "2024-06-01", LegRule::Section104);
        assert_eq!(s104.allowable_cost, dec!(30));
        assert_eq!(s104.gross_proceeds, dec!(39));
        assert_eq!(s104.net_proceeds, dec!(39));
        assert_eq!(s104.gain_or_loss, dec!(9));

        let pool = holding(&report, "ABC");
        assert_eq!(pool.quantity, dec!(7));
        assert_eq!(pool.total_cost, dec!(70));
    }

    #[test]
    fn bed_and_breakfast_leg_ignores_a_later_capital_return() {
        // The B&B leg prices off the raw 2024-06-15 buy with no cost offset:
        // unit (30*14 + 3)/30 = 14.1, cost 30*14.1 = 423. The 2024-07-01
        // CAPRETURN reduces only the pool that holds the S104 residue, never
        // the already-priced B&B leg (TCGA92/S106A consumed shares are not held
        // at the event date).
        let report = valued(
            "2024-01-01 BUY DEF 50 @ 10.00 GBP\n\
             2024-06-01 SELL DEF 50 @ 15.00 GBP FEES 5.00 GBP\n\
             2024-06-15 BUY DEF 30 @ 14.00 GBP FEES 3.00 GBP\n\
             2024-07-01 CAPRETURN DEF 50 TOTAL 100.00 GBP FEES 0.00 GBP\n",
        );

        let bnb = leg(&report, "2024-06-01", LegRule::BedAndBreakfast);
        assert_eq!(bnb.allowable_cost, dec!(423));
        assert_eq!(bnb.gross_proceeds, dec!(450));
        assert_eq!(bnb.net_proceeds, dec!(447));
        assert_eq!(bnb.gain_or_loss, dec!(24));
        assert_eq!(
            bnb.acquisition_date,
            Some(NaiveDate::from_ymd_opt(2024, 6, 15).expect("valid date"))
        );

        let s104 = leg(&report, "2024-06-01", LegRule::Section104);
        assert_eq!(s104.allowable_cost, dec!(200));
        assert_eq!(s104.gross_proceeds, dec!(300));
        assert_eq!(s104.net_proceeds, dec!(298));
        assert_eq!(s104.gain_or_loss, dec!(98));

        // The CAPRETURN net (100 - 0) reduces only the pool: 300 - 100 = 200.
        let pool = holding(&report, "DEF");
        assert_eq!(pool.quantity, dec!(30));
        assert_eq!(pool.total_cost, dec!(200));
    }

    #[test]
    fn same_day_leg_ignores_a_later_capital_return() {
        // A same-day disposal's cost is fixed at the disposal date: it prices
        // off that day's buy (40 of 100 @ 10 = 400) and is never adjusted by a
        // corporate action months later. The CAPRETURN reduces only the pool
        // that holds the 60-share residue (600 - 50 = 550).
        let report = valued(
            "2024-01-15 BUY ABC 100 @ 10.00 GBP\n\
             2024-01-15 SELL ABC 40 @ 9.00 GBP\n\
             2024-06-15 CAPRETURN ABC 60 TOTAL 50.00 GBP FEES 0.00 GBP\n",
        );

        let same_day = leg(&report, "2024-01-15", LegRule::SameDay);
        assert_eq!(same_day.allowable_cost, dec!(400));
        assert_eq!(same_day.gross_proceeds, dec!(360));
        assert_eq!(same_day.net_proceeds, dec!(360));
        assert_eq!(same_day.gain_or_loss, dec!(-40));

        let pool = holding(&report, "ABC");
        assert_eq!(pool.quantity, dec!(60));
        assert_eq!(pool.total_cost, dec!(550));
    }

    #[test]
    fn bed_and_breakfast_leg_unaffected_by_later_corporate_actions() {
        // Shares matched under bed-and-breakfast leave the Section 104 holding
        // (CG51560), so a later capital distribution or accumulation adjusts the
        // pool of shares still held (TCGA92/S110(8)(d)), never the matched legs.
        // B&B 30 shares price off the raw buy (30*14+3)/30 = 14.1 -> 423; the
        // 2024-07-01 CAPRETURN(40) + ACCUMULATION(60) net +20 lands in the 30
        // pooled shares (300 - 40 + 60 = 320).
        let report = valued(
            "2024-01-01 BUY DEF 50 @ 10.00 GBP\n\
             2024-06-01 SELL DEF 50 @ 15.00 GBP FEES 5.00 GBP\n\
             2024-06-15 BUY DEF 30 @ 14.00 GBP FEES 3.00 GBP\n\
             2024-07-01 CAPRETURN DEF 20 TOTAL 40.00 GBP FEES 0.00 GBP\n\
             2024-07-01 ACCUMULATION DEF 20 TOTAL 60.00 GBP TAX 0.00 GBP\n",
        );

        let bnb = leg(&report, "2024-06-01", LegRule::BedAndBreakfast);
        assert_eq!(bnb.allowable_cost, dec!(423));
        assert_eq!(bnb.gain_or_loss, dec!(24));
        let s104 = leg(&report, "2024-06-01", LegRule::Section104);
        assert_eq!(s104.allowable_cost, dec!(200));

        let pool = holding(&report, "DEF");
        assert_eq!(pool.quantity, dec!(30));
        assert_eq!(pool.total_cost, dec!(320));
    }

    #[test]
    fn same_day_leg_unaffected_by_later_corporate_actions() {
        // Same-day-matched shares also leave the Section 104 holding (CG51560);
        // the 2024-07-01 CAPRETURN(50) + ACCUMULATION(80) net +30 adjusts only
        // the 90 pooled shares (900 - 50 + 80 = 930), not the same-day leg
        // (20 @ 12 = 240).
        let report = valued(
            "2024-01-01 BUY GHI 100 @ 10.00 GBP\n\
             2024-06-01 BUY GHI 20 @ 12.00 GBP\n\
             2024-06-01 SELL GHI 30 @ 13.00 GBP\n\
             2024-07-01 CAPRETURN GHI 90 TOTAL 50.00 GBP FEES 0.00 GBP\n\
             2024-07-01 ACCUMULATION GHI 90 TOTAL 80.00 GBP TAX 0.00 GBP\n",
        );

        let same_day = leg(&report, "2024-06-01", LegRule::SameDay);
        assert_eq!(same_day.allowable_cost, dec!(240));
        assert_eq!(same_day.gain_or_loss, dec!(20));
        let s104 = leg(&report, "2024-06-01", LegRule::Section104);
        assert_eq!(s104.allowable_cost, dec!(100));

        let pool = holding(&report, "GHI");
        assert_eq!(pool.quantity, dec!(90));
        assert_eq!(pool.total_cost, dec!(930));
    }
}
