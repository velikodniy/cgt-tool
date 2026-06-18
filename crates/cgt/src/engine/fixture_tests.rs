//! Fixture-derived plan tests and the share-conservation invariant.
//!
//! Fixtures are the golden inputs in tests/inputs/. These tests assert leg
//! QUANTITIES and event structure only; monetary values are pinned by the
//! golden-file tests.

use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use std::sync::OnceLock;

use chrono::NaiveDate;
use rust_decimal::Decimal;
use rust_decimal_macros::dec;

use super::normalize::{EventId, EventKind, EventStream, normalize};
use super::plan::{DisposalPlan, MatchPlan, plan};
use crate::money::FxCache;

fn fx_cache() -> &'static FxCache {
    static CACHE: OnceLock<FxCache> = OnceLock::new();
    CACHE.get_or_init(|| crate::money::load_default_cache().expect("bundled FX rates load"))
}

fn fixtures_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../tests/inputs")
}

fn fixture_paths() -> Vec<PathBuf> {
    let mut paths: Vec<PathBuf> = fs::read_dir(fixtures_dir())
        .map(|entries| {
            entries
                .filter_map(Result::ok)
                .map(|entry| entry.path())
                .filter(|path| path.extension().is_some_and(|ext| ext == "cgt"))
                .collect()
        })
        .unwrap_or_default();
    paths.sort();
    paths
}

fn stream_and_plan(name: &str) -> (EventStream, MatchPlan) {
    let path = fixtures_dir().join(format!("{name}.cgt"));
    let content =
        fs::read_to_string(&path).unwrap_or_else(|e| panic!("cannot read {}: {e}", path.display()));
    let transactions =
        crate::dsl::parse(&content).unwrap_or_else(|e| panic!("{name}: parse failed: {e}"));
    let stream = normalize(&transactions, Some(fx_cache()))
        .unwrap_or_else(|e| panic!("{name}: normalize failed: {e}"));
    let match_plan = plan(&stream).unwrap_or_else(|e| panic!("{name}: plan failed: {e}"));
    (stream, match_plan)
}

fn date(s: &str) -> NaiveDate {
    NaiveDate::parse_from_str(s, "%Y-%m-%d").expect("valid test date")
}

fn event_id(stream: &EventStream, day: &str, ticker: &str, is_buy: bool) -> EventId {
    stream
        .events()
        .iter()
        .find(|event| {
            event.date == date(day)
                && event.ticker == ticker
                && match &event.kind {
                    EventKind::Buy(_) => is_buy,
                    EventKind::Sell(_) => !is_buy,
                    _ => false,
                }
        })
        .map(|event| event.id)
        .unwrap_or_else(|| {
            panic!(
                "no {} event for {ticker} on {day}",
                if is_buy { "buy" } else { "sell" }
            )
        })
}

fn disposal<'p>(
    match_plan: &'p MatchPlan,
    stream: &EventStream,
    day: &str,
    ticker: &str,
) -> &'p DisposalPlan {
    let sell = event_id(stream, day, ticker, false);
    match_plan
        .disposals
        .iter()
        .find(|disposal| disposal.sell == sell)
        .unwrap_or_else(|| panic!("no disposal plan for {ticker} on {day}"))
}

#[test]
fn same_day_reservation_fixture_plan() {
    // tests/inputs/SameDayReservation.cgt: Same Day priority over B&B per
    // TCGA92/S106A(9). 1 Feb: 30 B&B + 70 S104; 2 Feb: 50 Same Day.
    let (stream, match_plan) = stream_and_plan("SameDayReservation");
    assert_eq!(match_plan.disposals.len(), 2);

    let feb1 = disposal(&match_plan, &stream, "2024-02-01", "SNAP");
    assert!(feb1.same_day.is_none());
    assert_eq!(feb1.bed_and_breakfast.len(), 1);
    let leg = &feb1.bed_and_breakfast[0];
    assert_eq!(leg.buy, event_id(&stream, "2024-02-02", "SNAP", true));
    assert_eq!(leg.acquisition_date, date("2024-02-02"));
    assert_eq!(leg.quantity_at_sell_scale, dec!(30));
    assert_eq!(leg.quantity_at_buy_scale, dec!(30));
    assert_eq!(
        feb1.section_104.as_ref().map(|leg| leg.quantity),
        Some(dec!(70))
    );

    let feb2 = disposal(&match_plan, &stream, "2024-02-02", "SNAP");
    assert_eq!(
        feb2.same_day.as_ref().map(|leg| leg.quantity),
        Some(dec!(50))
    );
    assert!(feb2.bed_and_breakfast.is_empty());
    assert!(feb2.section_104.is_none());

    assert_eq!(
        match_plan
            .bnb_reservations
            .get(&event_id(&stream, "2024-02-02", "SNAP", true)),
        Some(&dec!(30))
    );
}

#[test]
fn future_consumption_fixture_plan() {
    // tests/inputs/single_pass_future_consumption.cgt: the February B&B
    // match fully reserves the 10 Feb buy; June drains the pool.
    let (stream, match_plan) = stream_and_plan("single_pass_future_consumption");
    assert_eq!(match_plan.disposals.len(), 2);

    let feb = disposal(&match_plan, &stream, "2024-02-01", "ABC");
    assert!(feb.same_day.is_none());
    assert_eq!(feb.bed_and_breakfast.len(), 1);
    assert_eq!(feb.bed_and_breakfast[0].quantity_at_sell_scale, dec!(5));
    assert_eq!(feb.bed_and_breakfast[0].quantity_at_buy_scale, dec!(5));
    assert_eq!(
        feb.bed_and_breakfast[0].buy,
        event_id(&stream, "2024-02-10", "ABC", true)
    );
    assert!(feb.section_104.is_none());

    let june = disposal(&match_plan, &stream, "2024-06-01", "ABC");
    assert!(june.bed_and_breakfast.is_empty());
    assert_eq!(
        june.section_104.as_ref().map(|leg| leg.quantity),
        Some(dec!(10))
    );

    assert_eq!(
        match_plan
            .bnb_reservations
            .get(&event_id(&stream, "2024-02-10", "ABC", true)),
        Some(&dec!(5))
    );
}

#[test]
fn bnb_split_fixture_plan_maps_ratio() {
    // tests/inputs/single_pass_bnb_split.cgt: SPLIT 2 inside the window —
    // 4 post-split shares cover 2 pre-split shares; remainder 3 from pool.
    let (stream, match_plan) = stream_and_plan("single_pass_bnb_split");
    assert_eq!(match_plan.disposals.len(), 1);

    let feb = disposal(&match_plan, &stream, "2024-02-01", "ABC");
    assert_eq!(feb.bed_and_breakfast.len(), 1);
    let leg = &feb.bed_and_breakfast[0];
    assert_eq!(leg.buy, event_id(&stream, "2024-02-20", "ABC", true));
    assert_eq!(leg.acquisition_date, date("2024-02-20"));
    assert_eq!(leg.quantity_at_sell_scale, dec!(2));
    assert_eq!(leg.quantity_at_buy_scale, dec!(4));
    assert_eq!(
        feb.section_104.as_ref().map(|leg| leg.quantity),
        Some(dec!(3))
    );
    assert_eq!(
        match_plan
            .bnb_reservations
            .get(&event_id(&stream, "2024-02-20", "ABC", true)),
        Some(&dec!(4))
    );
}

#[test]
fn bnb_boundary_fixture_plan() {
    // tests/inputs/BnBBoundary30Days.cgt: 1 Jul is D+30 (B&B), 2 Sep is
    // D+32 from 1 Aug (S104).
    let (stream, match_plan) = stream_and_plan("BnBBoundary30Days");
    assert_eq!(match_plan.disposals.len(), 2);

    let june = disposal(&match_plan, &stream, "2024-06-01", "ACME");
    assert_eq!(june.bed_and_breakfast.len(), 1);
    assert_eq!(
        june.bed_and_breakfast[0].buy,
        event_id(&stream, "2024-07-01", "ACME", true)
    );
    assert_eq!(june.bed_and_breakfast[0].quantity_at_sell_scale, dec!(100));
    assert!(june.section_104.is_none());

    let august = disposal(&match_plan, &stream, "2024-08-01", "ACME");
    assert!(august.bed_and_breakfast.is_empty());
    assert_eq!(
        august.section_104.as_ref().map(|leg| leg.quantity),
        Some(dec!(100))
    );
}

#[test]
fn interleaved_same_day_fixture_aggregates_to_single_events() {
    // tests/inputs/SameDayMergeInterleaved.cgt: three interleaved buys on
    // 2018-08-28 become ONE buy event; two sells on 2018-10-28 become ONE
    // sell event matched entirely from the pool.
    let (stream, match_plan) = stream_and_plan("SameDayMergeInterleaved");
    assert_eq!(stream.events().len(), 2);
    let EventKind::Buy(buy) = &stream.events()[0].kind else {
        panic!("first event must be the merged buy");
    };
    assert_eq!(buy.quantity, dec!(30));
    assert_eq!(buy.fees, dec!(16.5));

    assert_eq!(match_plan.disposals.len(), 1);
    let sell = disposal(&match_plan, &stream, "2018-10-28", "GB00B41YBW71");
    assert!(sell.same_day.is_none());
    assert!(sell.bed_and_breakfast.is_empty());
    assert_eq!(
        sell.section_104.as_ref().map(|leg| leg.quantity),
        Some(dec!(20))
    );
    assert!(match_plan.bnb_reservations.is_empty());
}

#[test]
fn share_conservation_invariant_over_all_fixtures() {
    let paths = fixture_paths();
    assert!(
        paths.len() >= 40,
        "fixture auto-discovery found too few .cgt files: {}",
        paths.len()
    );
    for path in paths {
        let name = path
            .file_stem()
            .and_then(|stem| stem.to_str())
            .unwrap_or("<non-utf8>")
            .to_string();
        let (stream, match_plan) = stream_and_plan(&name);
        check_conservation(&name, &stream, &match_plan);
    }
}

/// The invariant, checked independently of the planner's own bookkeeping:
/// every disposal is exactly covered by its legs; B&B reservations never
/// exceed any buy; day availability and pool quantities never go negative
/// when the plan is replayed over the stream.
fn check_conservation(name: &str, stream: &EventStream, match_plan: &MatchPlan) {
    let mut by_sell: HashMap<EventId, &DisposalPlan> = HashMap::new();
    let mut last_index: Option<usize> = None;
    for disposal in &match_plan.disposals {
        assert!(
            by_sell.insert(disposal.sell, disposal).is_none(),
            "{name}: duplicate disposal for sell event {:?}",
            disposal.sell
        );
        let index = disposal.sell.index();
        assert!(
            last_index.is_none_or(|previous| previous < index),
            "{name}: disposals out of stream order at {:?}",
            disposal.sell
        );
        last_index = Some(index);
    }

    // Per-disposal: legs positive, B&B legs well-formed, quantities conserve.
    for disposal in &match_plan.disposals {
        let sell_event = stream
            .get(disposal.sell)
            .unwrap_or_else(|| panic!("{name}: dangling sell id {:?}", disposal.sell));
        let EventKind::Sell(sell_trade) = &sell_event.kind else {
            panic!(
                "{name}: disposal {:?} does not reference a sell",
                disposal.sell
            );
        };
        if let Some(leg) = &disposal.same_day {
            assert!(leg.quantity > Decimal::ZERO, "{name}: zero same-day leg");
        }
        let mut previous_buy: Option<usize> = None;
        for leg in &disposal.bed_and_breakfast {
            assert!(
                leg.quantity_at_sell_scale > Decimal::ZERO
                    && leg.quantity_at_buy_scale > Decimal::ZERO,
                "{name}: zero B&B leg"
            );
            let buy_event = stream
                .get(leg.buy)
                .unwrap_or_else(|| panic!("{name}: dangling buy id {:?}", leg.buy));
            assert!(
                matches!(buy_event.kind, EventKind::Buy(_)),
                "{name}: B&B leg target is not a buy"
            );
            assert_eq!(
                buy_event.ticker, sell_event.ticker,
                "{name}: B&B leg crosses tickers"
            );
            assert_eq!(
                leg.acquisition_date, buy_event.date,
                "{name}: B&B acquisition date mismatch"
            );
            let days = (buy_event.date - sell_event.date).num_days();
            assert!(
                (1..=30).contains(&days),
                "{name}: B&B leg outside window: {days} days"
            );
            assert!(
                previous_buy.is_none_or(|previous| previous < leg.buy.index()),
                "{name}: B&B legs not chronological"
            );
            previous_buy = Some(leg.buy.index());
        }
        if let Some(leg) = &disposal.section_104 {
            assert!(leg.quantity > Decimal::ZERO, "{name}: zero S104 leg");
        }
        assert_eq!(
            disposal.matched_quantity(),
            sell_trade.quantity,
            "{name}: disposal on {} not exactly covered by legs",
            sell_event.date
        );
    }

    // Each buy's recorded B&B reservation equals the sum of every disposal
    // leg that draws on that buy, measured at the buy's share scale. Buys with
    // no B&B leg carry no reservation entry.
    let mut reserved_by_legs: HashMap<EventId, Decimal> = HashMap::new();
    for disposal in &match_plan.disposals {
        for leg in &disposal.bed_and_breakfast {
            *reserved_by_legs.entry(leg.buy).or_default() += leg.quantity_at_buy_scale;
        }
    }
    assert_eq!(
        reserved_by_legs.len(),
        match_plan.bnb_reservations.len(),
        "{name}: B&B reservation entries do not match the buys consumed by legs"
    );
    for (buy, leg_total) in &reserved_by_legs {
        assert_eq!(
            match_plan.bnb_reservations.get(buy),
            Some(leg_total),
            "{name}: reservation for buy {buy:?} disagrees with its legs"
        );
    }

    // Replay: reservations bounded by buys; day ledger and pool never negative.
    let mut pools: HashMap<&str, Decimal> = HashMap::new();
    let events = stream.events();
    let mut day_start = 0;
    while day_start < events.len() {
        let day_date = events[day_start].date;
        let day_len = events[day_start..]
            .iter()
            .take_while(|event| event.date == day_date)
            .count();
        let day = &events[day_start..day_start + day_len];

        let mut available: HashMap<&str, Decimal> = HashMap::new();
        for event in day {
            if let EventKind::Buy(trade) = &event.kind {
                let reserved = match_plan
                    .bnb_reservations
                    .get(&event.id)
                    .copied()
                    .unwrap_or(Decimal::ZERO);
                assert!(
                    reserved >= Decimal::ZERO && reserved <= trade.quantity,
                    "{name}: B&B reservations ({reserved}) exceed buy ({}) on {day_date}",
                    trade.quantity
                );
                available.insert(event.ticker.as_str(), trade.quantity - reserved);
            }
        }
        for event in day {
            if let EventKind::Sell(trade) = &event.kind {
                let Some(disposal) = by_sell.get(&event.id) else {
                    assert_eq!(
                        trade.quantity,
                        Decimal::ZERO,
                        "{name}: non-zero sell on {day_date} has no disposal plan"
                    );
                    continue;
                };
                if let Some(leg) = &disposal.same_day {
                    let day_quantity =
                        available.get_mut(event.ticker.as_str()).unwrap_or_else(|| {
                            panic!("{name}: same-day leg without same-day buy on {day_date}")
                        });
                    *day_quantity -= leg.quantity;
                    assert!(
                        *day_quantity >= Decimal::ZERO,
                        "{name}: day ledger overdrawn for {} on {day_date}",
                        event.ticker
                    );
                }
                if let Some(leg) = &disposal.section_104 {
                    let pool = pools.entry(event.ticker.as_str()).or_default();
                    *pool -= leg.quantity;
                    assert!(
                        *pool >= Decimal::ZERO,
                        "{name}: pool overdrawn for {} on {day_date}",
                        event.ticker
                    );
                }
            }
        }
        for event in day {
            if let EventKind::Buy(_) = &event.kind
                && let Some(leftover) = available.remove(event.ticker.as_str())
            {
                *pools.entry(event.ticker.as_str()).or_default() += leftover;
            }
        }
        for event in day {
            match &event.kind {
                EventKind::Split { ratio } => {
                    if let Some(pool) = pools.get_mut(event.ticker.as_str()) {
                        *pool *= *ratio;
                    }
                }
                EventKind::Unsplit { ratio } => {
                    if let Some(pool) = pools.get_mut(event.ticker.as_str())
                        && *ratio != Decimal::ZERO
                    {
                        *pool /= *ratio;
                    }
                }
                _ => {}
            }
        }
        day_start += day_len;
    }
    for (ticker, quantity) in pools {
        assert!(
            quantity >= Decimal::ZERO,
            "{name}: final pool negative for {ticker}: {quantity}"
        );
    }
}

#[test]
fn merged_holding_check_rejects_interleaved_overselling() {
    // Same-day sells of a ticker form one disposal (CG51560), so the
    // holding check applies to the merged day total: 150 sold against 100
    // held must error, even though each sell row individually fits when the
    // 60-share buy ten days later is matched by B&B.
    let input = "2024-01-01 BUY TTT 100 @ 1 GBP\n\
                 2024-06-03 SELL TTT 100 @ 1 GBP\n\
                 2024-06-03 BUY UUU 1 @ 1 GBP\n\
                 2024-06-03 SELL TTT 50 @ 1 GBP\n\
                 2024-06-13 BUY TTT 60 @ 1 GBP";
    let transactions = crate::dsl::parse(input).expect("test input parses");
    let stream = normalize(&transactions, None).expect("GBP-only input normalizes");
    let err = plan(&stream).expect_err("merged 150-share sell exceeds 100 held");
    match err {
        crate::CgtError::DisposalExceedsHolding {
            ticker,
            attempted,
            held,
            ..
        } => {
            assert_eq!(ticker, "TTT");
            assert_eq!(attempted, dec!(150));
            assert_eq!(held, dec!(100));
        }
        other => panic!("expected DisposalExceedsHolding, got {other}"),
    }
}
