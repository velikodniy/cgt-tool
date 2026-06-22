//! Input normalization: GBP conversion, stable date sort, day grouping, and
//! true (date, ticker, side) trade aggregation into an indexed event stream.
//!
//! Same-day buys/sells of one ticker aggregate into ONE event per side,
//! regardless of how the input rows are interleaved: same-day trades form a
//! single acquisition and a single disposal (CG51560), and the holding
//! check downstream applies to the merged day total.

use chrono::{Datelike, NaiveDate};
use rust_decimal::Decimal;

use crate::error::CgtError;
use crate::model::{Operation, Transaction};
use crate::money::{CurrencyAmount, FxCache};

/// Position of an event in its [`EventStream`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub(crate) struct EventId(usize);

impl EventId {
    /// Index into the stream's event list. Only the tests inspect raw
    /// positions; the engine addresses events through `after`/`get`.
    #[cfg(test)]
    pub(crate) fn index(self) -> usize {
        self.0
    }
}

/// One side (buy or sell) of a day's trading in one ticker, aggregated.
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct Trade {
    pub(crate) quantity: Decimal,
    /// Weighted-average unit price in GBP.
    pub(crate) price: Decimal,
    /// Total fees in GBP.
    pub(crate) fees: Decimal,
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) enum EventKind {
    /// All buys of one ticker on one day.
    Buy(Trade),
    /// All sells of one ticker on one day.
    Sell(Trade),
    Split {
        ratio: Decimal,
    },
    Unsplit {
        ratio: Decimal,
    },
    /// Value-domain pass-through: no effect on quantity matching.
    Dividend {
        total_value: Decimal,
        tax_paid: Decimal,
    },
    /// Value-domain pass-through: no effect on quantity matching.
    Accumulation {
        quantity: Decimal,
        total_value: Decimal,
        tax_paid: Decimal,
    },
    /// Value-domain pass-through: no effect on quantity matching.
    CapitalReturn {
        quantity: Decimal,
        total_value: Decimal,
        fees: Decimal,
    },
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct Event {
    pub(crate) id: EventId,
    pub(crate) date: NaiveDate,
    pub(crate) ticker: String,
    pub(crate) kind: EventKind,
}

/// Chronological, GBP-converted, aggregated event stream.
///
/// Canonical intra-day order: value events (CAPRETURN, ACCUMULATION,
/// DIVIDEND), then aggregated buys, then aggregated sells, then
/// SPLIT/UNSPLIT. Corporate-action cost adjustments therefore apply before
/// the day's trades, and splits scale quantities only after the day's
/// trades have matched. Relative input order is preserved within each
/// group (stable sort).
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct EventStream {
    events: Vec<Event>,
}

impl EventStream {
    pub(crate) fn events(&self) -> &[Event] {
        &self.events
    }

    /// Events strictly after `id`, in stream order.
    pub(crate) fn after(&self, id: EventId) -> &[Event] {
        self.events.get(id.0 + 1..).unwrap_or_default()
    }

    /// Event by id.
    pub(crate) fn get(&self, id: EventId) -> Option<&Event> {
        self.events.get(id.0)
    }
}

struct RawEvent {
    date: NaiveDate,
    ticker: String,
    kind: EventKind,
}

/// Normalize parsed transactions into the engine's event stream.
pub(crate) fn normalize(
    transactions: &[Transaction],
    fx_cache: Option<&FxCache>,
) -> Result<EventStream, CgtError> {
    let mut raw = transactions
        .iter()
        .map(|tx| {
            Ok(RawEvent {
                date: tx.date,
                ticker: tx.ticker.clone(),
                kind: kind_to_gbp(&tx.operation, tx.date, fx_cache)?,
            })
        })
        .collect::<Result<Vec<_>, CgtError>>()?;

    // Stable sort by date only: input order is preserved within a day.
    raw.sort_by_key(|event| event.date);

    let mut events = Vec::new();
    for day in raw.chunk_by(|a, b| a.date == b.date) {
        push_day(&mut events, day);
    }

    Ok(EventStream { events })
}

/// Convert one operation to GBP: every monetary field converts at the
/// transaction date; quantities and ratios pass through unchanged.
fn kind_to_gbp(
    operation: &Operation<CurrencyAmount>,
    date: NaiveDate,
    fx_cache: Option<&FxCache>,
) -> Result<EventKind, CgtError> {
    match operation {
        Operation::Buy {
            amount,
            price,
            fees,
        } => Ok(EventKind::Buy(Trade {
            quantity: *amount,
            price: amount_to_gbp(price, date, fx_cache)?,
            fees: amount_to_gbp(fees, date, fx_cache)?,
        })),
        Operation::Sell {
            amount,
            price,
            fees,
        } => Ok(EventKind::Sell(Trade {
            quantity: *amount,
            price: amount_to_gbp(price, date, fx_cache)?,
            fees: amount_to_gbp(fees, date, fx_cache)?,
        })),
        Operation::Dividend {
            total_value,
            tax_paid,
        } => Ok(EventKind::Dividend {
            total_value: amount_to_gbp(total_value, date, fx_cache)?,
            tax_paid: amount_to_gbp(tax_paid, date, fx_cache)?,
        }),
        Operation::Accumulation {
            amount,
            total_value,
            tax_paid,
        } => Ok(EventKind::Accumulation {
            quantity: *amount,
            total_value: amount_to_gbp(total_value, date, fx_cache)?,
            tax_paid: amount_to_gbp(tax_paid, date, fx_cache)?,
        }),
        Operation::CapReturn {
            amount,
            total_value,
            fees,
        } => Ok(EventKind::CapitalReturn {
            quantity: *amount,
            total_value: amount_to_gbp(total_value, date, fx_cache)?,
            fees: amount_to_gbp(fees, date, fx_cache)?,
        }),
        Operation::Split { ratio } => Ok(EventKind::Split { ratio: *ratio }),
        Operation::Unsplit { ratio } => Ok(EventKind::Unsplit { ratio: *ratio }),
    }
}

/// Convert an amount to GBP: GBP passes through without consulting the
/// cache; for non-GBP amounts a missing cache or missing rate is a
/// `MissingFxRate` error naming the currency and the transaction's month.
fn amount_to_gbp(
    amount: &CurrencyAmount,
    date: NaiveDate,
    fx_cache: Option<&FxCache>,
) -> Result<Decimal, CgtError> {
    if amount.is_gbp() {
        return Ok(amount.amount);
    }
    let cache = fx_cache.ok_or_else(|| CgtError::MissingFxRate {
        currency: amount.code().to_string(),
        year: date.year(),
        month: date.month(),
    })?;
    Ok(amount.to_gbp(date, cache)?)
}

/// Aggregate one day's raw events and append them in canonical order.
fn push_day(events: &mut Vec<Event>, day: &[RawEvent]) {
    let Some(first) = day.first() else {
        return;
    };
    let date = first.date;

    let mut leading: Vec<(String, EventKind)> = Vec::new();
    let mut buys: Vec<(String, Trade)> = Vec::new();
    let mut sells: Vec<(String, Trade)> = Vec::new();
    let mut trailing: Vec<(String, EventKind)> = Vec::new();

    for event in day {
        match &event.kind {
            EventKind::Buy(trade) => merge_into(&mut buys, &event.ticker, trade),
            EventKind::Sell(trade) => merge_into(&mut sells, &event.ticker, trade),
            EventKind::Split { .. } | EventKind::Unsplit { .. } => {
                trailing.push((event.ticker.clone(), event.kind.clone()));
            }
            EventKind::Dividend { .. }
            | EventKind::Accumulation { .. }
            | EventKind::CapitalReturn { .. } => {
                leading.push((event.ticker.clone(), event.kind.clone()));
            }
        }
    }

    // Canonical value-event order (CAPRETURN, ACCUMULATION, DIVIDEND), so the
    // stream does not depend on input row order.
    leading.sort_by_key(|(_, kind)| value_event_rank(kind));
    for (ticker, kind) in leading {
        push_event(events, date, ticker, kind);
    }
    for (ticker, trade) in buys {
        push_event(events, date, ticker, EventKind::Buy(trade));
    }
    for (ticker, trade) in sells {
        push_event(events, date, ticker, EventKind::Sell(trade));
    }
    for (ticker, kind) in trailing {
        push_event(events, date, ticker, kind);
    }
}

/// Canonical same-day order for value events (lower applies first).
fn value_event_rank(kind: &EventKind) -> u8 {
    match kind {
        EventKind::CapitalReturn { .. } => 0,
        EventKind::Accumulation { .. } => 1,
        EventKind::Dividend { .. } => 2,
        _ => 3,
    }
}

/// Fold a trade into its ticker's bucket, preserving first-occurrence order.
fn merge_into(side: &mut Vec<(String, Trade)>, ticker: &str, trade: &Trade) {
    match side
        .iter_mut()
        .find(|(existing, _)| existing.as_str() == ticker)
    {
        Some((_, existing)) => merge_trades(existing, trade),
        None => side.push((ticker.to_string(), trade.clone())),
    }
}

/// Pairwise fold, re-deriving the weighted-average price at each step.
/// The operation order is load-bearing for output equivalence: rust_decimal
/// is path-dependent, so do not replace this with sum-of-products followed
/// by a single division.
fn merge_trades(current: &mut Trade, next: &Trade) {
    let total = (current.quantity * current.price) + (next.quantity * next.price);
    current.quantity += next.quantity;
    if current.quantity != Decimal::ZERO {
        current.price = total / current.quantity;
    }
    current.fees += next.fees;
}

fn push_event(events: &mut Vec<Event>, date: NaiveDate, ticker: String, kind: EventKind) {
    let id = EventId(events.len());
    events.push(Event {
        id,
        date,
        ticker,
        kind,
    });
}

#[cfg(test)]
mod tests {
    use rust_decimal::Decimal;
    use rust_decimal_macros::dec;

    use super::{EventKind, EventStream, normalize};
    use crate::error::CgtError;
    use crate::money::{Currency, FxCache, RateEntry, RateKey, RateSource};

    fn stream(input: &str, fx_cache: Option<&FxCache>) -> EventStream {
        let transactions = crate::dsl::parse(input).expect("test DSL parses");
        normalize(&transactions, fx_cache).expect("test input normalizes")
    }

    fn usd_cache(year: i32, month: u32, rate_per_gbp: Decimal) -> FxCache {
        let mut cache = FxCache::new();
        cache.insert(RateEntry {
            key: RateKey::new(Currency::USD, year, month),
            rate_per_gbp,
            source: RateSource::Bundled { period: None },
            minor_units: 2,
            symbol: None,
        });
        cache
    }

    #[test]
    fn empty_input_yields_empty_stream() {
        let stream = stream("", None);
        assert!(stream.events().is_empty());
    }

    #[test]
    fn interleaved_same_day_trades_aggregate_per_side() {
        // Interleaved buy/sell/buy of one ticker on one day becomes ONE buy
        // event and ONE sell event, regardless of row adjacency.
        let stream = stream(
            "2024-03-01 BUY ABC 10 @ 10.00 GBP FEES 1.00 GBP\n\
             2024-03-01 SELL ABC 5 @ 11.00 GBP\n\
             2024-03-01 BUY ABC 20 @ 12.00 GBP FEES 2.00 GBP\n",
            None,
        );
        let events = stream.events();
        assert_eq!(events.len(), 2);
        for (position, event) in events.iter().enumerate() {
            assert_eq!(event.id.index(), position);
            assert_eq!(event.ticker, "ABC");
        }
        let EventKind::Buy(buy) = &events[0].kind else {
            panic!("first event must be the merged buy, got {:?}", events[0]);
        };
        assert_eq!(buy.quantity, dec!(30));
        assert_eq!(buy.price, dec!(340) / dec!(30));
        assert_eq!(buy.fees, dec!(3));
        let EventKind::Sell(sell) = &events[1].kind else {
            panic!("second event must be the merged sell, got {:?}", events[1]);
        };
        assert_eq!(sell.quantity, dec!(5));
        assert_eq!(sell.price, dec!(11));
        assert_eq!(sell.fees, Decimal::ZERO);
    }

    #[test]
    fn three_way_merge_folds_pairwise() {
        // Three buys merge pairwise, not via a single sum-then-divide.
        let stream = stream(
            "2018-08-28 BUY GB00B41YBW71 10 @ 8 FEES 2\n\
             2018-10-28 SELL GB00B41YBW71 10 @ 7 FEES 12.5\n\
             2018-08-28 BUY GB00B41YBW71 10 @ 10 FEES 2\n\
             2018-10-28 SELL GB00B41YBW71 10 @ 9 FEES 2\n\
             2018-08-28 BUY GB00B41YBW71 10 @ 5 FEES 12.5\n",
            None,
        );
        let events = stream.events();
        assert_eq!(events.len(), 2);
        let EventKind::Buy(buy) = &events[0].kind else {
            panic!("expected merged buy first, got {:?}", events[0]);
        };
        let price_after_two = (dec!(80) + dec!(100)) / dec!(20);
        let expected_price = ((dec!(20) * price_after_two) + dec!(50)) / dec!(30);
        assert_eq!(buy.quantity, dec!(30));
        assert_eq!(buy.price, expected_price);
        assert_eq!(buy.fees, dec!(16.5));
        let EventKind::Sell(sell) = &events[1].kind else {
            panic!("expected merged sell second, got {:?}", events[1]);
        };
        assert_eq!(sell.quantity, dec!(20));
        assert_eq!(sell.price, dec!(8));
        assert_eq!(sell.fees, dec!(14.5));
    }

    #[test]
    fn stable_sort_and_canonical_intra_day_order() {
        let stream = stream(
            "2024-05-02 BUY ZZZ 1 @ 1.00 GBP\n\
             2024-05-01 UNSPLIT AAA RATIO 3\n\
             2024-05-01 SELL AAA 5 @ 2.00 GBP\n\
             2024-05-01 DIVIDEND AAA TOTAL 10 TAX 1\n\
             2024-05-01 BUY AAA 5 @ 2.00 GBP\n\
             2024-05-01 SPLIT AAA RATIO 2\n\
             2024-05-02 BUY AAA 1 @ 1.00 GBP\n",
            None,
        );
        let events = stream.events();
        assert_eq!(events.len(), 7);
        // Day 1: value events, buys, sells, then splits in input order.
        assert!(matches!(
            &events[0].kind,
            EventKind::Dividend { total_value, tax_paid }
                if *total_value == dec!(10) && *tax_paid == dec!(1)
        ));
        assert!(matches!(&events[1].kind, EventKind::Buy(_)));
        assert!(matches!(&events[2].kind, EventKind::Sell(_)));
        assert!(matches!(
            &events[3].kind,
            EventKind::Unsplit { ratio } if *ratio == dec!(3)
        ));
        assert!(matches!(
            &events[4].kind,
            EventKind::Split { ratio } if *ratio == dec!(2)
        ));
        // Day 2: stable sort keeps ZZZ (input line 1) before AAA.
        assert_eq!(events[5].ticker, "ZZZ");
        assert_eq!(events[6].ticker, "AAA");
        assert!(matches!(&events[5].kind, EventKind::Buy(_)));
        assert!(matches!(&events[6].kind, EventKind::Buy(_)));
    }

    #[test]
    fn value_events_pass_through_with_fx_conversion() {
        let cache = usd_cache(2024, 3, dec!(1.25));
        let stream = stream(
            "2024-03-15 DIVIDEND ABC TOTAL 100.00 USD TAX 10.00 USD\n\
             2024-03-15 ACCUMULATION ABC 7 TOTAL 50.00 USD TAX 2.50 USD\n\
             2024-03-15 CAPRETURN ABC 7 TOTAL 25.00 USD FEES 2.50 USD\n",
            Some(&cache),
        );
        // Emitted in canonical order (CAPRETURN, ACCUMULATION, DIVIDEND),
        // not the input order.
        let events = stream.events();
        assert_eq!(events.len(), 3);
        assert!(matches!(
            &events[0].kind,
            EventKind::CapitalReturn { quantity, total_value, fees }
                if *quantity == dec!(7) && *total_value == dec!(20) && *fees == dec!(2)
        ));
        assert!(matches!(
            &events[1].kind,
            EventKind::Accumulation { quantity, total_value, tax_paid }
                if *quantity == dec!(7) && *total_value == dec!(40) && *tax_paid == dec!(2)
        ));
        assert!(matches!(
            &events[2].kind,
            EventKind::Dividend { total_value, tax_paid }
                if *total_value == dec!(80) && *tax_paid == dec!(8)
        ));
    }

    #[test]
    fn value_event_order_is_independent_of_input_order() {
        let a = stream(
            "2024-03-15 ACCUMULATION ABC 7 TOTAL 50.00 GBP TAX 0\n\
             2024-03-15 CAPRETURN ABC 7 TOTAL 25.00 GBP FEES 0\n",
            None,
        );
        let b = stream(
            "2024-03-15 CAPRETURN ABC 7 TOTAL 25.00 GBP FEES 0\n\
             2024-03-15 ACCUMULATION ABC 7 TOTAL 50.00 GBP TAX 0\n",
            None,
        );
        assert_eq!(a.events(), b.events());
    }

    #[test]
    fn usd_trade_converts_at_monthly_rate() {
        let cache = usd_cache(2024, 3, dec!(1.25));
        let stream = stream(
            "2024-03-15 BUY ABC 10 @ 150.00 USD FEES 5.00 USD\n",
            Some(&cache),
        );
        let EventKind::Buy(buy) = &stream.events()[0].kind else {
            panic!("expected buy event");
        };
        assert_eq!(buy.quantity, dec!(10));
        assert_eq!(buy.price, dec!(120));
        assert_eq!(buy.fees, dec!(4));
    }

    #[test]
    fn gbp_amounts_bypass_fx_cache() {
        // GBP must pass through even with NO cache at all.
        let stream = stream(
            "2024-03-15 BUY ABC 10 @ 150.00 GBP FEES 5.00 GBP\n\
             2024-03-16 DIVIDEND ABC TOTAL 10.00 GBP\n",
            None,
        );
        assert_eq!(stream.events().len(), 2);
    }

    #[test]
    fn missing_fx_rate_without_cache_names_currency_and_month() {
        let transactions =
            crate::dsl::parse("2024-03-15 BUY ABC 10 @ 150.00 USD\n").expect("parses");
        let err = normalize(&transactions, None).expect_err("must fail without cache");
        assert!(matches!(
            err,
            CgtError::MissingFxRate { ref currency, year: 2024, month: 3 } if currency == "USD"
        ));
    }

    #[test]
    fn missing_fx_rate_with_cache_missing_month_names_currency_and_month() {
        // Cache exists but lacks the transaction's month.
        let cache = usd_cache(2024, 2, dec!(1.25));
        let transactions =
            crate::dsl::parse("2024-03-15 BUY ABC 10 @ 150.00 USD\n").expect("parses");
        let err = normalize(&transactions, Some(&cache)).expect_err("must fail for missing month");
        assert!(matches!(
            err,
            CgtError::MissingFxRate { ref currency, year: 2024, month: 3 } if currency == "USD"
        ));
    }
}
