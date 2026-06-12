//! Phase 1 of the engine: quantity-only match planning over the normalized
//! event stream.
//!
//! Reproduces the legacy matcher's quantity semantics (cgt-core/src/matcher/):
//! pre-cascade holding verification (CG51590; mod.rs:380-401), Same Day
//! (TCGA92/S105(1); same_day.rs), Bed & Breakfast with same-day reservations
//! and split-ratio mapping (TCGA92/S106A; bed_and_breakfast.rs), then
//! Section 104 (section104.rs). NO COSTS are computed here; legs carry
//! quantities and event references only.

use std::collections::HashMap;

use rust_decimal::Decimal;

use super::normalize::{Event, EventId, EventKind, EventStream, Trade};
use crate::error::CgtError;

/// B&B window: acquisitions 1..=30 days after the disposal (CG51560).
const BNB_WINDOW_DAYS: i64 = 30;

/// Same-day matched quantity for one disposal.
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct SameDayLeg {
    pub(crate) quantity: Decimal,
}

/// Bed & Breakfast matched quantity for one disposal, tied to its buy event.
///
/// Two scales because SPLIT/UNSPLIT events between the sell and the buy
/// change the share denomination (bed_and_breakfast.rs:100-110):
/// `quantity_at_buy_scale = quantity_at_sell_scale * cumulative_ratio`.
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct BnbLeg {
    pub(crate) buy: EventId,
    pub(crate) acquisition_date: chrono::NaiveDate,
    /// Quantity in the disposal's share scale (before window splits).
    pub(crate) quantity_at_sell_scale: Decimal,
    /// Quantity in the acquisition's share scale (after window splits).
    pub(crate) quantity_at_buy_scale: Decimal,
}

/// Section 104 matched quantity for one disposal.
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct Section104Leg {
    pub(crate) quantity: Decimal,
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct DisposalPlan {
    pub(crate) sell: EventId,
    pub(crate) same_day: Option<SameDayLeg>,
    /// Chronological by construction (earliest window buy first).
    pub(crate) bed_and_breakfast: Vec<BnbLeg>,
    pub(crate) section_104: Option<Section104Leg>,
}

impl DisposalPlan {
    /// Total matched quantity at the disposal's share scale.
    pub(crate) fn matched_quantity(&self) -> Decimal {
        let same_day = self
            .same_day
            .as_ref()
            .map_or(Decimal::ZERO, |leg| leg.quantity);
        let bnb: Decimal = self
            .bed_and_breakfast
            .iter()
            .map(|leg| leg.quantity_at_sell_scale)
            .sum();
        let section_104 = self
            .section_104
            .as_ref()
            .map_or(Decimal::ZERO, |leg| leg.quantity);
        same_day + bnb + section_104
    }
}

/// Output of Phase 1: per-disposal legs plus per-buy B&B reservations.
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct MatchPlan {
    /// One entry per merged sell event with non-zero quantity, chronological.
    pub(crate) disposals: Vec<DisposalPlan>,
    /// Per merged buy event: total quantity (at the buy's share scale)
    /// reserved by B&B matches from earlier disposals.
    pub(crate) bnb_reservations: HashMap<EventId, Decimal>,
}

/// Build the quantity-only match plan for a normalized stream.
pub(crate) fn plan(stream: &EventStream) -> Result<MatchPlan, CgtError> {
    let mut planner = Planner::new(stream);
    let events = stream.events();
    let mut day_start = 0;
    while day_start < events.len() {
        let date = events[day_start].date;
        let day_len = events[day_start..]
            .iter()
            .take_while(|event| event.date == date)
            .count();
        planner.process_day(&events[day_start..day_start + day_len])?;
        day_start += day_len;
    }
    Ok(MatchPlan {
        disposals: planner.disposals,
        bnb_reservations: planner.bnb_reservations,
    })
}

struct Planner<'s> {
    stream: &'s EventStream,
    /// Per-ticker Section 104 pool quantity.
    pools: HashMap<String, Decimal>,
    /// Day ledger for the day in progress: per ticker, merged buy quantity
    /// still available after B&B reservations and same-day consumption.
    day_available: HashMap<String, Decimal>,
    /// B&B reservations against future merged buys (buy share scale).
    bnb_reservations: HashMap<EventId, Decimal>,
    /// Same-day shields: per (date, ticker), sell quantity not yet protected
    /// from B&B consumption of that date's buys (bed_and_breakfast.rs:50-81).
    same_day_reservations: HashMap<(chrono::NaiveDate, String), Decimal>,
    disposals: Vec<DisposalPlan>,
}

impl<'s> Planner<'s> {
    fn new(stream: &'s EventStream) -> Self {
        Self {
            stream,
            pools: HashMap::new(),
            day_available: HashMap::new(),
            bnb_reservations: HashMap::new(),
            same_day_reservations: HashMap::new(),
            disposals: Vec::new(),
        }
    }

    /// Process one day, replicating the legacy day loop (matcher/mod.rs:96-182):
    /// buys enter the day ledger, sells run the matching cascade, leftover
    /// buy quantity pools, SPLIT/UNSPLIT scale the pool LAST.
    fn process_day(&mut self, day: &[Event]) -> Result<(), CgtError> {
        self.day_available.clear();

        for event in day {
            if let EventKind::Buy(trade) = &event.kind {
                let reserved = self
                    .bnb_reservations
                    .get(&event.id)
                    .copied()
                    .unwrap_or(Decimal::ZERO);
                if reserved > trade.quantity {
                    // Defensive legacy guard (matcher/mod.rs:128-134).
                    return Err(CgtError::BnbReservationExceedsBuy {
                        ticker: event.ticker.clone(),
                        date: event.date,
                    });
                }
                self.day_available
                    .insert(event.ticker.clone(), trade.quantity - reserved);
            }
        }

        for event in day {
            if let EventKind::Sell(trade) = &event.kind {
                self.process_sell(event, trade)?;
            }
        }

        for event in day {
            if let EventKind::Buy(_) = &event.kind {
                let leftover = self
                    .day_available
                    .insert(event.ticker.clone(), Decimal::ZERO)
                    .unwrap_or(Decimal::ZERO);
                if leftover > Decimal::ZERO {
                    *self.pools.entry(event.ticker.clone()).or_default() += leftover;
                }
            }
        }

        for event in day {
            match &event.kind {
                EventKind::Split { ratio } => {
                    if let Some(pool) = self.pools.get_mut(&event.ticker) {
                        *pool *= *ratio;
                    }
                }
                EventKind::Unsplit { ratio } => {
                    if let Some(pool) = self.pools.get_mut(&event.ticker)
                        && *ratio != Decimal::ZERO
                    {
                        *pool /= *ratio;
                    }
                }
                _ => {}
            }
        }

        Ok(())
    }

    fn process_sell(&mut self, sell: &Event, trade: &Trade) -> Result<(), CgtError> {
        // Zero-quantity disposals produce no legs and no disposal entry
        // (legacy emits no MatchResult: same_day.rs:36-40,
        // bed_and_breakfast.rs:174-177, section104.rs:31-34).
        if trade.quantity == Decimal::ZERO {
            return Ok(());
        }

        // Pre-cascade holding verification (CG51590; matcher/mod.rs:380-401):
        // the disposal must be covered by today's unconsumed buy quantity
        // plus the S104 pool. B&B never rescues an uncovered disposal.
        let ledger_held = self
            .day_available
            .get(sell.ticker.as_str())
            .copied()
            .unwrap_or(Decimal::ZERO);
        let pool_held = self
            .pools
            .get(sell.ticker.as_str())
            .copied()
            .unwrap_or(Decimal::ZERO);
        let total_held = ledger_held + pool_held;
        if trade.quantity > total_held {
            return Err(CgtError::DisposalExceedsHolding {
                ticker: sell.ticker.clone(),
                date: sell.date,
                attempted: trade.quantity,
                held: total_held,
                ledger: ledger_held,
                pool: pool_held,
            });
        }

        let mut remaining = trade.quantity;
        let mut disposal = DisposalPlan {
            sell: sell.id,
            same_day: None,
            bed_and_breakfast: Vec::new(),
            section_104: None,
        };

        // 1. Same Day (TCGA92/S105(1); same_day.rs:30-64).
        if ledger_held > Decimal::ZERO {
            let matched = remaining.min(ledger_held);
            self.day_available
                .insert(sell.ticker.clone(), ledger_held - matched);
            disposal.same_day = Some(SameDayLeg { quantity: matched });
            remaining -= matched;
        }

        // 2. Bed & Breakfast (TCGA92/S106A; bed_and_breakfast.rs:154-260).
        self.match_bed_and_breakfast(sell, &mut remaining, &mut disposal);

        // 3. Section 104 remainder (section104.rs:13-62).
        if remaining > Decimal::ZERO
            && let Some(pool) = self.pools.get_mut(sell.ticker.as_str())
            && *pool > Decimal::ZERO
        {
            let matched = remaining.min(*pool);
            *pool -= matched;
            remaining -= matched;
            disposal.section_104 = Some(Section104Leg { quantity: matched });
        }

        // Post-cascade exhaustiveness errors (matcher/mod.rs:434-447).
        // Defensive: the pre-cascade check makes these unreachable, but the
        // legacy cascade keeps them and so does the rewrite.
        if remaining > Decimal::ZERO {
            if remaining == trade.quantity {
                return Err(CgtError::NoPriorAcquisitions {
                    ticker: sell.ticker.clone(),
                    date: sell.date,
                    attempted: trade.quantity,
                });
            }
            return Err(CgtError::DisposalPartiallyUnmatched {
                ticker: sell.ticker.clone(),
                date: sell.date,
                attempted: trade.quantity,
                matched: trade.quantity - remaining,
                unmatched: remaining,
            });
        }

        self.disposals.push(disposal);
        Ok(())
    }

    /// Match the remainder against acquisitions 1..=30 days after the sell,
    /// chronologically, tracking SPLIT/UNSPLIT scale changes across the
    /// window (bed_and_breakfast.rs:154-260).
    fn match_bed_and_breakfast(
        &mut self,
        sell: &Event,
        remaining: &mut Decimal,
        disposal: &mut DisposalPlan,
    ) {
        let stream = self.stream;
        let mut cumulative_ratio_effect = Decimal::ONE;

        for event in stream.after(sell.id) {
            if *remaining <= Decimal::ZERO {
                break;
            }
            let days_diff = (event.date - sell.date).num_days();
            if days_diff <= 0 {
                // Same-day events are the Same Day rule's domain
                // (bed_and_breakfast.rs:195-198).
                continue;
            }
            if days_diff > BNB_WINDOW_DAYS {
                // Date-sorted stream: nothing later can be in the window.
                // (Legacy breaks only on same-ticker events past the window,
                // bed_and_breakfast.rs:188-202 — outcome-identical.)
                break;
            }
            if event.ticker != sell.ticker {
                continue;
            }
            match &event.kind {
                EventKind::Split { ratio } => {
                    cumulative_ratio_effect *= *ratio;
                }
                EventKind::Unsplit { ratio } => {
                    if *ratio != Decimal::ZERO {
                        cumulative_ratio_effect /= *ratio;
                    }
                }
                EventKind::Buy(buy) => {
                    if cumulative_ratio_effect == Decimal::ZERO {
                        // A zero SPLIT ratio leaves nothing to map back to
                        // the sell's scale. Legacy panics on this division
                        // (bed_and_breakfast.rs:105); validation rejects
                        // non-positive ratios, so this is unreachable for
                        // validated input. Skip to stay panic-free.
                        continue;
                    }
                    let available_at_buy_time = self.available_for_bnb(event, buy.quantity);
                    if available_at_buy_time <= Decimal::ZERO {
                        continue;
                    }
                    // Legacy arithmetic shape (bed_and_breakfast.rs:100-110):
                    // divide availability into the sell scale BEFORE the min,
                    // then multiply back; affects fractional-share rounding.
                    let available_at_sell_time = available_at_buy_time / cumulative_ratio_effect;
                    let matched_at_sell = (*remaining).min(available_at_sell_time);
                    let matched_at_buy = matched_at_sell * cumulative_ratio_effect;

                    disposal.bed_and_breakfast.push(BnbLeg {
                        buy: event.id,
                        acquisition_date: event.date,
                        quantity_at_sell_scale: matched_at_sell,
                        quantity_at_buy_scale: matched_at_buy,
                    });
                    *remaining -= matched_at_sell;
                    *self.bnb_reservations.entry(event.id).or_default() += matched_at_buy;
                }
                _ => {}
            }
        }
    }

    /// Quantity of a window buy available to B&B after earlier B&B
    /// reservations and the Same Day shield for the buy's date
    /// (bed_and_breakfast.rs:50-81).
    fn available_for_bnb(&mut self, buy_event: &Event, buy_quantity: Decimal) -> Decimal {
        let already_reserved = self
            .bnb_reservations
            .get(&buy_event.id)
            .copied()
            .unwrap_or(Decimal::ZERO);
        let available_before_same_day = buy_quantity - already_reserved;
        if available_before_same_day <= Decimal::ZERO {
            return Decimal::ZERO;
        }

        // Same Day priority over B&B (TCGA92/S106A(9), S105(1)): per
        // (date, ticker), reserve the total same-day sell quantity once,
        // then decrement as B&B consumes buys on that date. With true
        // aggregation that total is the merged sell event's quantity, which
        // equals the legacy per-transaction sum (bed_and_breakfast.rs:21-34).
        let total_same_day_sells = sell_quantity_on(self.stream, buy_event.date, &buy_event.ticker);
        let reservation_remaining = self
            .same_day_reservations
            .entry((buy_event.date, buy_event.ticker.clone()))
            .or_insert(total_same_day_sells);

        let reserve_now =
            available_before_same_day.min((*reservation_remaining).max(Decimal::ZERO));
        *reservation_remaining -= reserve_now;

        available_before_same_day - reserve_now
    }
}

/// Total sell quantity for (date, ticker) — the Same Day claim that shields
/// that date's buys from B&B (bed_and_breakfast.rs:21-34).
fn sell_quantity_on(stream: &EventStream, date: chrono::NaiveDate, ticker: &str) -> Decimal {
    stream
        .events()
        .iter()
        .filter(|event| event.date == date && event.ticker == ticker)
        .filter_map(|event| match &event.kind {
            EventKind::Sell(trade) => Some(trade.quantity),
            _ => None,
        })
        .sum()
}

#[cfg(test)]
mod tests {
    use chrono::NaiveDate;
    use rust_decimal::Decimal;
    use rust_decimal_macros::dec;

    use super::{MatchPlan, plan};
    use crate::engine::normalize::{EventId, EventKind, EventStream, normalize};
    use crate::error::CgtError;
    use crate::model::{Operation, Transaction};
    use crate::money::CurrencyAmount;

    fn stream_and_plan(input: &str) -> (EventStream, MatchPlan) {
        let transactions = crate::dsl::parse(input).expect("test DSL parses");
        let stream = normalize(&transactions, None).expect("test input normalizes");
        let match_plan = plan(&stream).expect("test input plans");
        (stream, match_plan)
    }

    fn plan_err(input: &str) -> CgtError {
        let transactions = crate::dsl::parse(input).expect("test DSL parses");
        let stream = normalize(&transactions, None).expect("test input normalizes");
        plan(&stream).expect_err("planning must fail")
    }

    fn sell_id(stream: &EventStream, date: &str, ticker: &str) -> EventId {
        stream
            .events()
            .iter()
            .find(|event| {
                event.date.to_string() == date
                    && event.ticker == ticker
                    && matches!(event.kind, EventKind::Sell(_))
            })
            .map(|event| event.id)
            .expect("sell event present")
    }

    fn buy_id(stream: &EventStream, date: &str, ticker: &str) -> EventId {
        stream
            .events()
            .iter()
            .find(|event| {
                event.date.to_string() == date
                    && event.ticker == ticker
                    && matches!(event.kind, EventKind::Buy(_))
            })
            .map(|event| event.id)
            .expect("buy event present")
    }

    #[test]
    fn pool_buy_then_later_sell_matches_section_104() {
        let (stream, match_plan) = stream_and_plan(
            "2024-01-01 BUY ABC 10 @ 100.00 GBP\n\
             2024-06-01 SELL ABC 4 @ 120.00 GBP\n",
        );
        assert_eq!(match_plan.disposals.len(), 1);
        let disposal = &match_plan.disposals[0];
        assert_eq!(disposal.sell, sell_id(&stream, "2024-06-01", "ABC"));
        assert!(disposal.same_day.is_none());
        assert_eq!(
            disposal.section_104.as_ref().map(|leg| leg.quantity),
            Some(dec!(4))
        );
        assert_eq!(disposal.matched_quantity(), dec!(4));
        assert!(match_plan.bnb_reservations.is_empty());
    }

    #[test]
    fn same_day_takes_priority_then_pool_covers_remainder() {
        let (_, match_plan) = stream_and_plan(
            "2024-01-01 BUY ABC 10 @ 10.00 GBP\n\
             2024-03-01 BUY ABC 5 @ 12.00 GBP\n\
             2024-03-01 SELL ABC 8 @ 13.00 GBP\n",
        );
        assert_eq!(match_plan.disposals.len(), 1);
        let disposal = &match_plan.disposals[0];
        assert_eq!(
            disposal.same_day.as_ref().map(|leg| leg.quantity),
            Some(dec!(5))
        );
        assert_eq!(
            disposal.section_104.as_ref().map(|leg| leg.quantity),
            Some(dec!(3))
        );
    }

    #[test]
    fn interleaved_same_day_trades_plan_as_one_disposal() {
        let (_, match_plan) = stream_and_plan(
            "2024-03-01 BUY ABC 10 @ 10.00 GBP\n\
             2024-03-01 SELL ABC 5 @ 11.00 GBP\n\
             2024-03-01 BUY ABC 20 @ 12.00 GBP\n",
        );
        assert_eq!(match_plan.disposals.len(), 1);
        let disposal = &match_plan.disposals[0];
        assert_eq!(
            disposal.same_day.as_ref().map(|leg| leg.quantity),
            Some(dec!(5))
        );
        assert!(disposal.section_104.is_none());
    }

    #[test]
    fn zero_holding_disposal_fails_before_cascade() {
        // CG51590: a later buy within 30 days does NOT rescue a disposal of
        // shares the taxpayer does not hold.
        let err = plan_err(
            "2024-02-01 SELL ABC 10 @ 5.00 GBP\n\
             2024-02-10 BUY ABC 10 @ 4.00 GBP\n",
        );
        assert!(matches!(
            err,
            CgtError::DisposalExceedsHolding {
                ref ticker,
                attempted,
                held,
                ledger,
                pool,
                ..
            } if ticker == "ABC"
                && attempted == dec!(10)
                && held == Decimal::ZERO
                && ledger == Decimal::ZERO
                && pool == Decimal::ZERO
        ));
    }

    #[test]
    fn oversell_against_partial_pool_reports_held_breakdown() {
        let err = plan_err(
            "2024-01-01 BUY ABC 5 @ 10.00 GBP\n\
             2024-06-01 SELL ABC 8 @ 12.00 GBP\n",
        );
        assert!(matches!(
            err,
            CgtError::DisposalExceedsHolding {
                attempted,
                held,
                ledger,
                pool,
                ..
            } if attempted == dec!(8)
                && held == dec!(5)
                && ledger == Decimal::ZERO
                && pool == dec!(5)
        ));
    }

    #[test]
    fn split_scales_pool_after_same_date_sell() {
        // REPLICATES CURRENT BEHAVIOR: splits apply after trades on their
        // date (matcher/mod.rs:170-173). The spec's D4 adjudication of
        // same-date SPLIT/SELL ordering is deferred to Milestone D sign-off.
        let (_, match_plan) = stream_and_plan(
            "2024-01-01 BUY ABC 10 @ 10.00 GBP\n\
             2024-02-01 SPLIT ABC RATIO 2\n\
             2024-02-01 SELL ABC 5 @ 6.00 GBP\n\
             2024-06-01 SELL ABC 10 @ 6.00 GBP\n",
        );
        assert_eq!(match_plan.disposals.len(), 2);
        // 1 Feb sells 5 from the pre-split pool of 10; the residue (5)
        // doubles to 10, which exactly covers the June sell.
        assert_eq!(
            match_plan.disposals[0]
                .section_104
                .as_ref()
                .map(|leg| leg.quantity),
            Some(dec!(5))
        );
        assert_eq!(
            match_plan.disposals[1]
                .section_104
                .as_ref()
                .map(|leg| leg.quantity),
            Some(dec!(10))
        );
    }

    #[test]
    fn unsplit_scales_pool_down() {
        let (_, match_plan) = stream_and_plan(
            "2024-01-01 BUY ABC 10 @ 10.00 GBP\n\
             2024-02-01 UNSPLIT ABC RATIO 2\n\
             2024-06-01 SELL ABC 5 @ 30.00 GBP\n",
        );
        assert_eq!(match_plan.disposals.len(), 1);
        assert_eq!(
            match_plan.disposals[0]
                .section_104
                .as_ref()
                .map(|leg| leg.quantity),
            Some(dec!(5))
        );
    }

    #[test]
    fn capreturn_and_accumulation_are_quantity_neutral() {
        let (_, match_plan) = stream_and_plan(
            "2024-01-01 BUY ABC 10 @ 10.00 GBP\n\
             2024-03-01 CAPRETURN ABC 10 TOTAL 20.00 GBP\n\
             2024-04-01 ACCUMULATION ABC 10 TOTAL 5.00 GBP\n\
             2024-06-01 SELL ABC 10 @ 12.00 GBP\n",
        );
        assert_eq!(match_plan.disposals.len(), 1);
        assert_eq!(
            match_plan.disposals[0]
                .section_104
                .as_ref()
                .map(|leg| leg.quantity),
            Some(dec!(10))
        );
    }

    #[test]
    fn zero_quantity_sell_produces_no_disposal() {
        // Not constructible via the DSL/validation path; built directly to
        // pin the legacy no-MatchResult behavior for zero-quantity sells.
        let transactions = vec![Transaction {
            date: NaiveDate::from_ymd_opt(2024, 1, 5).expect("valid date"),
            ticker: "ABC".to_string(),
            operation: Operation::Sell {
                amount: Decimal::ZERO,
                price: CurrencyAmount::default(),
                fees: CurrencyAmount::default(),
            },
        }];
        let stream = normalize(&transactions, None).expect("normalizes");
        let match_plan = plan(&stream).expect("plans without error");
        assert!(match_plan.disposals.is_empty());
    }

    #[test]
    fn bnb_matches_day_30_boundary_and_leaves_pool_untouched() {
        // 1 Jun -> 1 Jul is days_diff 30: inside the window (CG51560).
        // The August sell then drains the pool, proving B&B never touched it.
        let (stream, match_plan) = stream_and_plan(
            "2024-01-15 BUY ACME 100 @ 10.00 GBP\n\
             2024-06-01 SELL ACME 100 @ 15.00 GBP\n\
             2024-07-01 BUY ACME 100 @ 14.00 GBP\n\
             2024-08-15 SELL ACME 100 @ 16.00 GBP\n",
        );
        assert_eq!(match_plan.disposals.len(), 2);
        let june = &match_plan.disposals[0];
        assert!(june.same_day.is_none());
        assert!(june.section_104.is_none());
        assert_eq!(june.bed_and_breakfast.len(), 1);
        let leg = &june.bed_and_breakfast[0];
        assert_eq!(leg.buy, buy_id(&stream, "2024-07-01", "ACME"));
        assert_eq!(leg.acquisition_date.to_string(), "2024-07-01");
        assert_eq!(leg.quantity_at_sell_scale, dec!(100));
        assert_eq!(leg.quantity_at_buy_scale, dec!(100));
        assert_eq!(
            match_plan
                .bnb_reservations
                .get(&buy_id(&stream, "2024-07-01", "ACME")),
            Some(&dec!(100))
        );
        let august = &match_plan.disposals[1];
        assert!(august.bed_and_breakfast.is_empty());
        assert_eq!(
            august.section_104.as_ref().map(|leg| leg.quantity),
            Some(dec!(100))
        );
    }

    #[test]
    fn bnb_window_excludes_day_31() {
        // 1 Jun -> 2 Jul is days_diff 31: outside the window, falls to S104.
        let (_, match_plan) = stream_and_plan(
            "2024-01-15 BUY ACME 100 @ 10.00 GBP\n\
             2024-06-01 SELL ACME 100 @ 15.00 GBP\n\
             2024-07-02 BUY ACME 100 @ 14.00 GBP\n",
        );
        assert_eq!(match_plan.disposals.len(), 1);
        let disposal = &match_plan.disposals[0];
        assert!(disposal.bed_and_breakfast.is_empty());
        assert_eq!(
            disposal.section_104.as_ref().map(|leg| leg.quantity),
            Some(dec!(100))
        );
        assert!(match_plan.bnb_reservations.is_empty());
    }

    #[test]
    fn bnb_matches_chronologically_earliest_first() {
        let (stream, match_plan) = stream_and_plan(
            "2024-01-01 BUY ABC 100 @ 10.00 GBP\n\
             2024-02-01 SELL ABC 50 @ 12.00 GBP\n\
             2024-02-05 BUY ABC 20 @ 11.00 GBP\n\
             2024-02-15 BUY ABC 40 @ 11.50 GBP\n",
        );
        let disposal = &match_plan.disposals[0];
        assert_eq!(disposal.bed_and_breakfast.len(), 2);
        assert_eq!(
            disposal.bed_and_breakfast[0].buy,
            buy_id(&stream, "2024-02-05", "ABC")
        );
        assert_eq!(
            disposal.bed_and_breakfast[0].quantity_at_sell_scale,
            dec!(20)
        );
        assert_eq!(
            disposal.bed_and_breakfast[1].buy,
            buy_id(&stream, "2024-02-15", "ABC")
        );
        assert_eq!(
            disposal.bed_and_breakfast[1].quantity_at_sell_scale,
            dec!(30)
        );
        assert!(disposal.section_104.is_none());
    }

    #[test]
    fn same_day_reservation_shields_future_buy_from_bnb() {
        // TCGA92/S106A(9): the 2 Feb same-day claim (30) is reserved before
        // the 1 Feb disposal may B&B-consume the 2 Feb buy of 50.
        let (stream, match_plan) = stream_and_plan(
            "2024-01-01 BUY SNAP 100 @ 10.00 GBP\n\
             2024-02-01 SELL SNAP 60 @ 12.00 GBP\n\
             2024-02-02 BUY SNAP 50 @ 11.00 GBP\n\
             2024-02-02 SELL SNAP 30 @ 11.50 GBP\n",
        );
        assert_eq!(match_plan.disposals.len(), 2);
        let feb1 = &match_plan.disposals[0];
        assert_eq!(feb1.bed_and_breakfast.len(), 1);
        assert_eq!(feb1.bed_and_breakfast[0].quantity_at_sell_scale, dec!(20));
        assert_eq!(
            feb1.section_104.as_ref().map(|leg| leg.quantity),
            Some(dec!(40))
        );
        let feb2 = &match_plan.disposals[1];
        assert_eq!(
            feb2.same_day.as_ref().map(|leg| leg.quantity),
            Some(dec!(30))
        );
        assert!(feb2.bed_and_breakfast.is_empty());
        assert!(feb2.section_104.is_none());
        assert_eq!(
            match_plan
                .bnb_reservations
                .get(&buy_id(&stream, "2024-02-02", "SNAP")),
            Some(&dec!(20))
        );
    }

    #[test]
    fn earlier_disposals_reserve_future_buys_before_later_disposals() {
        let (stream, match_plan) = stream_and_plan(
            "2024-01-01 BUY ABC 100 @ 10.00 GBP\n\
             2024-02-01 SELL ABC 30 @ 12.00 GBP\n\
             2024-02-03 SELL ABC 30 @ 12.00 GBP\n\
             2024-02-10 BUY ABC 40 @ 11.00 GBP\n",
        );
        assert_eq!(match_plan.disposals.len(), 2);
        let first = &match_plan.disposals[0];
        assert_eq!(first.bed_and_breakfast[0].quantity_at_sell_scale, dec!(30));
        assert!(first.section_104.is_none());
        let second = &match_plan.disposals[1];
        assert_eq!(second.bed_and_breakfast[0].quantity_at_sell_scale, dec!(10));
        assert_eq!(
            second.section_104.as_ref().map(|leg| leg.quantity),
            Some(dec!(20))
        );
        assert_eq!(
            match_plan
                .bnb_reservations
                .get(&buy_id(&stream, "2024-02-10", "ABC")),
            Some(&dec!(40))
        );
    }

    #[test]
    fn split_in_window_maps_quantities_between_scales() {
        // SPLIT 2 between sell and buy: 8 post-split shares cover 4
        // pre-split shares (division-then-min shape,
        // bed_and_breakfast.rs:100-110).
        let (stream, match_plan) = stream_and_plan(
            "2024-01-01 BUY ABC 20 @ 100.00 GBP\n\
             2024-02-01 SELL ABC 10 @ 120.00 GBP\n\
             2024-02-10 SPLIT ABC RATIO 2\n\
             2024-02-20 BUY ABC 8 @ 60.00 GBP\n",
        );
        let disposal = &match_plan.disposals[0];
        let leg = &disposal.bed_and_breakfast[0];
        assert_eq!(leg.quantity_at_sell_scale, dec!(4));
        assert_eq!(leg.quantity_at_buy_scale, dec!(8));
        assert_eq!(
            leg.quantity_at_buy_scale,
            leg.quantity_at_sell_scale * dec!(2)
        );
        assert_eq!(
            disposal.section_104.as_ref().map(|leg| leg.quantity),
            Some(dec!(6))
        );
        assert_eq!(
            match_plan
                .bnb_reservations
                .get(&buy_id(&stream, "2024-02-20", "ABC")),
            Some(&dec!(8))
        );
    }

    #[test]
    fn unsplit_in_window_maps_quantities_between_scales() {
        // UNSPLIT 2 between sell and buy: 4 post-unsplit shares cover 8
        // pre-unsplit shares.
        let (_, match_plan) = stream_and_plan(
            "2024-01-01 BUY ABC 20 @ 10.00 GBP\n\
             2024-02-01 SELL ABC 10 @ 12.00 GBP\n\
             2024-02-10 UNSPLIT ABC RATIO 2\n\
             2024-02-20 BUY ABC 4 @ 25.00 GBP\n",
        );
        let disposal = &match_plan.disposals[0];
        let leg = &disposal.bed_and_breakfast[0];
        assert_eq!(leg.quantity_at_sell_scale, dec!(8));
        assert_eq!(leg.quantity_at_buy_scale, dec!(4));
        assert_eq!(
            disposal.section_104.as_ref().map(|leg| leg.quantity),
            Some(dec!(2))
        );
    }

    #[test]
    fn capreturn_in_window_is_ignored_by_bnb() {
        let (stream, match_plan) = stream_and_plan(
            "2024-01-01 BUY ABC 10 @ 10.00 GBP\n\
             2024-02-01 SELL ABC 5 @ 12.00 GBP\n\
             2024-02-05 CAPRETURN ABC 5 TOTAL 10.00 GBP\n\
             2024-02-10 BUY ABC 5 @ 11.00 GBP\n",
        );
        let disposal = &match_plan.disposals[0];
        assert_eq!(disposal.bed_and_breakfast.len(), 1);
        assert_eq!(
            disposal.bed_and_breakfast[0].buy,
            buy_id(&stream, "2024-02-10", "ABC")
        );
        assert_eq!(
            disposal.bed_and_breakfast[0].quantity_at_sell_scale,
            dec!(5)
        );
    }
}
