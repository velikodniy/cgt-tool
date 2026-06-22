//! Golden-value tests over every fixture in tests/inputs/.
//!
//! Every fixture's engine output must serde-equal its checked-in JSON golden.
//! Six fixtures additionally carry hand-stated value pins (a disposal's
//! allowable cost no longer leaks basis from a later corporate action); those
//! pins are independent value anchors, computed without reference to the
//! golden files they sit alongside.

use std::fs;
use std::path::PathBuf;
use std::sync::OnceLock;

use rust_decimal::Decimal;
use serde_json::Value;

use crate::config::Config;
use crate::money::FxCache;

fn fx_cache() -> &'static FxCache {
    static CACHE: OnceLock<FxCache> = OnceLock::new();
    CACHE.get_or_init(|| crate::money::load_default_cache().expect("bundled FX rates load"))
}

fn config() -> Config {
    Config::embedded().expect("embedded config loads")
}

fn fixtures_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../tests/inputs")
}

fn golden_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../tests/json")
}

fn fixture_names() -> Vec<String> {
    let mut names: Vec<String> = fs::read_dir(fixtures_dir())
        .expect("read fixtures dir")
        .filter_map(Result::ok)
        .map(|entry| entry.path())
        .filter(|path| path.extension().is_some_and(|ext| ext == "cgt"))
        .filter_map(|path| path.file_stem().and_then(|s| s.to_str()).map(String::from))
        .collect();
    names.sort();
    names
}

/// Run the new engine over a fixture and serialize the report to JSON.
fn run_engine(name: &str) -> Value {
    let path = fixtures_dir().join(format!("{name}.cgt"));
    let content = fs::read_to_string(&path).expect("read fixture");
    let transactions = crate::dsl::parse(&content).expect("fixture parses");
    let report =
        crate::calculate(&transactions, None, Some(fx_cache()), &config()).expect("report builds");
    serde_json::to_value(&report).expect("report serializes")
}

fn golden(name: &str) -> Value {
    let path = golden_dir().join(format!("{name}.json"));
    let content = fs::read_to_string(&path).expect("read golden");
    serde_json::from_str(&content).expect("golden parses")
}

/// Parse a JSON money/quantity string field into a Decimal.
fn money(value: &Value, field: &str) -> Decimal {
    value[field]
        .as_str()
        .unwrap_or_else(|| panic!("field {field} is not a string in {value}"))
        .parse()
        .unwrap_or_else(|e| panic!("field {field} is not a decimal: {e}"))
}

#[test]
fn every_fixture_serde_equals_its_golden() {
    let names = fixture_names();
    assert_eq!(names.len(), 49, "fixture count drifted: {}", names.len());

    for name in &names {
        let new = run_engine(name);
        let old = golden(name);
        assert_eq!(new, old, "{name}: engine output diverged from golden");
    }
}

/// Every year's published totals must foot against the rounded rows actually
/// displayed (F8/F11): no sub-penny dust can flip a bucket or unbalance an
/// identity, because the legs carry 2dp-rounded gains and costs.
#[test]
fn year_totals_foot_against_rounded_rows() {
    for name in &fixture_names() {
        let report = run_engine(name);
        for year in report["tax_years"].as_array().expect("tax_years") {
            let mut sum_gain = Decimal::ZERO;
            let mut sum_loss = Decimal::ZERO;
            let mut sum_proceeds = Decimal::ZERO;
            let mut sum_cost = Decimal::ZERO;
            for disposal in year["disposals"].as_array().expect("disposals") {
                sum_proceeds += money(disposal, "gross_proceeds");
                let mut disposal_net = Decimal::ZERO;
                for leg in disposal["matches"].as_array().expect("matches") {
                    sum_cost += money(leg, "allowable_cost");
                    disposal_net += money(leg, "gain_or_loss");
                }
                if disposal_net > Decimal::ZERO {
                    sum_gain += disposal_net;
                } else if disposal_net < Decimal::ZERO {
                    sum_loss += disposal_net.abs();
                }
            }
            let total_gain = money(year, "total_gain");
            let total_loss = money(year, "total_loss");
            let net_gain = money(year, "net_gain");
            let exempt = money(year, "exempt_amount");
            let taxable = money(year, "taxable_gain");
            assert_eq!(total_gain, sum_gain, "{name}: total_gain vs summed rows");
            assert_eq!(total_loss, sum_loss, "{name}: total_loss vs summed rows");
            assert_eq!(
                net_gain,
                total_gain - total_loss,
                "{name}: net_gain identity"
            );
            assert_eq!(
                taxable,
                (net_gain - exempt).max(Decimal::ZERO),
                "{name}: taxable_gain identity"
            );
            assert_eq!(
                money(year, "gross_proceeds"),
                sum_proceeds,
                "{name}: year gross_proceeds vs summed disposal rows"
            );
            assert_eq!(
                money(year, "total_allowable_cost"),
                sum_cost,
                "{name}: total_allowable_cost vs summed leg rows"
            );
        }
    }
}

/// Disposal legs for the given date/ticker, as (rule, allowable_cost, gain).
fn legs_on(report: &Value, date: &str, ticker: &str) -> Vec<(String, Decimal, Decimal)> {
    let mut out = Vec::new();
    for year in report["tax_years"].as_array().expect("tax_years") {
        for disposal in year["disposals"].as_array().expect("disposals") {
            if disposal["date"] == date && disposal["ticker"] == ticker {
                for leg in disposal["matches"].as_array().expect("matches") {
                    out.push((
                        leg["rule"].as_str().expect("rule").to_string(),
                        money(leg, "allowable_cost"),
                        money(leg, "gain_or_loss"),
                    ));
                }
            }
        }
    }
    out
}

/// The single disposal on `date` for `ticker`.
fn disposal_on<'a>(report: &'a Value, date: &str, ticker: &str) -> &'a Value {
    for year in report["tax_years"].as_array().expect("tax_years") {
        for disposal in year["disposals"].as_array().expect("disposals") {
            if disposal["date"] == date && disposal["ticker"] == ticker {
                return disposal;
            }
        }
    }
    panic!("no disposal on {date} for {ticker}")
}

fn holding<'a>(report: &'a Value, ticker: &str) -> &'a Value {
    report["holdings"]
        .as_array()
        .expect("holdings")
        .iter()
        .find(|h| h["ticker"] == ticker)
        .unwrap_or_else(|| panic!("no holding for {ticker}"))
}

fn year_total(report: &Value, period: &str, field: &str) -> Decimal {
    let year = report["tax_years"]
        .as_array()
        .expect("tax_years")
        .iter()
        .find(|y| y["period"] == period)
        .unwrap_or_else(|| panic!("no year {period}"));
    money(year, field)
}

fn d(s: &str) -> Decimal {
    s.parse().expect("decimal literal")
}

/// A disposal of `ticker` on `date` matched entirely from the Section 104 pool
/// with the given allowable cost and gain.
fn assert_s104(report: &Value, date: &str, ticker: &str, cost: &str, gain: &str) {
    let legs = legs_on(report, date, ticker);
    assert_eq!(legs.len(), 1, "{date} {ticker}: expected one leg");
    assert_eq!(legs[0].0, "Section104", "{date} {ticker}: rule");
    assert_eq!(legs[0].1, d(cost), "{date} {ticker}: cost");
    assert_eq!(legs[0].2, d(gain), "{date} {ticker}: gain");
}

fn assert_holding(report: &Value, ticker: &str, quantity: &str, total_cost: &str) {
    let h = holding(report, ticker);
    assert_eq!(money(h, "quantity"), d(quantity), "{ticker}: quantity");
    assert_eq!(
        money(h, "total_cost"),
        d(total_cost),
        "{ticker}: total_cost"
    );
}

#[test]
fn asset_events_not_full_sale_drops_retroactive_basis() {
    let report = run_engine("AssetEventsNotFullSale");
    assert_s104(&report, "2019-09-01", "FOOBAR", "500.00", "25.00");
    assert_s104(&report, "2020-07-01", "FOOBAR", "436.00", "64.00");
    assert_holding(&report, "FOOBAR", "20", "1774.00");
}

#[test]
fn asset_events_not_full_sale2_drops_retroactive_basis() {
    let report = run_engine("AssetEventsNotFullSale2");
    assert_s104(&report, "2019-09-01", "FOOBAR", "500.00", "25.00");
    assert_s104(&report, "2020-07-01", "FOOBAR", "1744.00", "256.00");
    assert_holding(&report, "FOOBAR", "5", "436.00");
}

#[test]
fn with_asset_events_bb_prices_off_raw_buy() {
    let report = run_engine("WithAssetEventsBB");
    let legs = legs_on(&report, "2019-11-05", "GB00B3TYHH97");
    assert_eq!(legs.len(), 2);
    assert_eq!(legs[0].0, "BedAndBreakfast");
    assert_eq!(legs[0].1, d("3805.80"));
    assert_eq!(legs[0].2, d("72.35"));
    assert_eq!(legs[1].0, "Section104");
    assert_eq!(legs[1].1, d("3669.60"));
    assert_eq!(legs[1].2, d("208.56"));
    // Disposal gain raw 280.905 renders 280.91 away from zero.
    let disposal = disposal_on(&report, "2019-11-05", "GB00B3TYHH97");
    let net: Decimal = disposal["matches"]
        .as_array()
        .expect("matches")
        .iter()
        .map(|leg| money(leg, "gain_or_loss"))
        .sum();
    assert_eq!(net, d("280.91"));
    assert_eq!(year_total(&report, "2019/20", "net_gain"), d("280.91"));
    assert_holding(&report, "GB00B3TYHH97", "20", "3685.41");
}

#[test]
fn accumulation_dividend_adjusts_each_pool_disposal() {
    let report = run_engine("AccumulationDividend");
    assert_s104(&report, "2023-06-01", "ACCFUND", "502.50", "92.50");
    assert_s104(&report, "2023-12-01", "ACCFUND", "527.50", "17.50");
    assert_holding(&report, "ACCFUND", "0", "0.00");
}

#[test]
fn with_asset_events_same_day_drops_retroactive_basis() {
    let report = run_engine("WithAssetEventsSameDay");
    let legs = legs_on(&report, "2019-11-05", "GB00B3TYHH97");
    assert_eq!(legs.len(), 2);
    assert_eq!(legs[0].0, "SameDay");
    assert_eq!(legs[0].1, d("3886.40"));
    assert_eq!(legs[0].2, d("-8.25"));
    assert_eq!(legs[1].0, "Section104");
    assert_eq!(legs[1].1, d("3669.60"));
    assert_eq!(legs[1].2, d("208.56"));
    // Disposal net raw 200.305 renders 200.31 away from zero.
    assert_eq!(year_total(&report, "2019/20", "net_gain"), d("200.31"));
    assert_eq!(year_total(&report, "2019/20", "total_gain"), d("200.31"));
    assert_holding(&report, "GB00B3TYHH97", "20", "3685.41");
}

#[test]
fn synthetic_complex_same_day_legs_lose_leaked_offset() {
    let report = run_engine("SyntheticComplex");
    // Pre-corporate-action same-day legs: cost rises, gain flips negative.
    let may2020 = legs_on(&report, "2020-05-15", "ACME");
    assert_eq!(may2020.len(), 1);
    assert_eq!(may2020[0].0, "SameDay");
    assert_eq!(may2020[0].1, d("4765.96"));
    assert_eq!(may2020[0].2, d("-38.21"));
    let may2022 = legs_on(&report, "2022-05-15", "ACME");
    assert_eq!(may2022.len(), 1);
    assert_eq!(may2022[0].0, "SameDay");
    assert_eq!(may2022[0].1, d("12103.00"));
    assert_eq!(may2022[0].2, d("-28.94"));
    // Post-corporate-action pool legs share the corrected unit cost.
    for date in ["2023-09-01", "2023-09-02", "2023-09-03", "2023-09-04"] {
        let legs = legs_on(&report, date, "ACME");
        assert_eq!(legs.len(), 1, "{date}: one S104 leg");
        assert_eq!(legs[0].0, "Section104");
        assert_eq!(legs[0].1, d("2810.70"), "{date}: corrected unit cost");
    }
    assert_eq!(
        money(holding(&report, "ACME"), "total_cost"),
        d("258579.93")
    );
}
