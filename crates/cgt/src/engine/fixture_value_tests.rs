//! Golden-value tests over every fixture in tests/inputs/.
//!
//! The new engine renders money 2dp midpoint-away-from-zero; the checked-in
//! goldens were rendered with banker's rounding. So unaffected fixtures are
//! compared to their golden with a 0.01 money tolerance: that absorbs the
//! last-digit rounding-policy difference on a handful of fixtures while still
//! catching any real divergence (> 0.01). Six fixtures whose values genuinely
//! changed (a disposal's allowable cost no longer leaks basis from a later
//! corporate action) assert hand-stated new values instead of their golden.

use std::collections::HashSet;
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

/// The six fixtures whose values changed: a disposal's allowable cost no longer
/// inherits basis from a corporate action that occurs after the disposal.
fn adjudicated() -> HashSet<&'static str> {
    HashSet::from([
        "AssetEventsNotFullSale",
        "AssetEventsNotFullSale2",
        "WithAssetEventsBB",
        "AccumulationDividend",
        "WithAssetEventsSameDay",
        "SyntheticComplex",
    ])
}

const MONEY_TOLERANCE: Decimal = Decimal::from_parts(1, 0, 0, false, 2); // 0.01

#[test]
fn every_fixture_is_compared_or_adjudicated() {
    let names = fixture_names();
    let adjudicated = adjudicated();
    let mut compared = 0usize;
    let mut asserted = 0usize;

    for name in &names {
        if adjudicated.contains(name.as_str()) {
            asserted += 1;
        } else {
            compare_to_golden(name);
            compared += 1;
        }
    }

    assert_eq!(
        compared + asserted,
        46,
        "fixture count drifted: {compared} compared + {asserted} adjudicated"
    );
    assert_eq!(
        asserted,
        adjudicated.len(),
        "every adjudicated fixture seen"
    );
}

/// Compare the new engine's report to the golden over the fields the golden
/// carries: per-leg allowable_cost & gain_or_loss, per-disposal gross_proceeds
/// & proceeds, per-year total_gain/total_loss/net_gain, per-holding quantity
/// (exact) & total_cost. Money fields pass within 0.01; quantities are exact.
/// Only keys present in the golden are compared.
fn compare_to_golden(name: &str) {
    let new = run_engine(name);
    let old = golden(name);

    let new_years = new["tax_years"].as_array().expect("new tax_years array");
    let old_years = old["tax_years"].as_array().expect("golden tax_years array");
    assert_eq!(
        new_years.len(),
        old_years.len(),
        "{name}: tax-year count differs"
    );

    for (new_year, old_year) in new_years.iter().zip(old_years) {
        assert_eq!(
            new_year["period"], old_year["period"],
            "{name}: tax-year period differs"
        );
        let period = old_year["period"].as_str().unwrap_or("?");
        for field in ["total_gain", "total_loss", "net_gain"] {
            if old_year.get(field).is_some() {
                close(
                    name,
                    &format!("{period} {field}"),
                    money(new_year, field),
                    money(old_year, field),
                );
            }
        }

        let new_disposals = new_year["disposals"].as_array().expect("new disposals");
        let old_disposals = old_year["disposals"].as_array().expect("golden disposals");
        assert_eq!(
            new_disposals.len(),
            old_disposals.len(),
            "{name}: disposal count differs in {period}"
        );

        // Same-date disposals may be emitted in a different order than the
        // golden (the engine sorts by ticker; the golden preserves input
        // order), so disposals match on (date, ticker) with a sequence number
        // for genuine duplicates, as the equivalence harness does.
        let new_keyed = key_disposals(new_disposals);
        for (old_d, key) in keyed_iter(old_disposals) {
            let new_d: &Value = new_keyed
                .get(&key)
                .copied()
                .unwrap_or_else(|| panic!("{name}: no new disposal for {key} in {period}"));
            let where_ = format!("{period} disposal {key}");
            for field in ["gross_proceeds", "proceeds"] {
                if old_d.get(field).is_some() {
                    close(
                        name,
                        &format!("{where_} {field}"),
                        money(new_d, field),
                        money(old_d, field),
                    );
                }
            }

            let new_legs = new_d["matches"].as_array().expect("new matches");
            let old_legs = old_d["matches"].as_array().expect("golden matches");
            assert_eq!(
                new_legs.len(),
                old_legs.len(),
                "{name}: {where_}: leg count differs"
            );
            for (j, (new_leg, old_leg)) in new_legs.iter().zip(old_legs).enumerate() {
                assert_eq!(
                    new_leg["rule"], old_leg["rule"],
                    "{name}: {where_} leg {j}: rule differs"
                );
                for field in ["allowable_cost", "gain_or_loss"] {
                    if old_leg.get(field).is_some() {
                        close(
                            name,
                            &format!("{where_} leg {j} {field}"),
                            money(new_leg, field),
                            money(old_leg, field),
                        );
                    }
                }
            }
        }
    }

    let new_holdings = new["holdings"].as_array().expect("new holdings");
    let old_holdings = old["holdings"].as_array().expect("golden holdings");
    assert_eq!(
        new_holdings.len(),
        old_holdings.len(),
        "{name}: holding count differs"
    );
    for (new_h, old_h) in new_holdings.iter().zip(old_holdings) {
        assert_eq!(
            new_h["ticker"], old_h["ticker"],
            "{name}: holding ticker differs"
        );
        let ticker = old_h["ticker"].as_str().unwrap_or("?");
        assert_eq!(
            money(new_h, "quantity"),
            money(old_h, "quantity"),
            "{name}: holding {ticker}: quantity differs"
        );
        if old_h.get("total_cost").is_some() {
            close(
                name,
                &format!("holding {ticker} total_cost"),
                money(new_h, "total_cost"),
                money(old_h, "total_cost"),
            );
        }
    }
}

/// Stable key for one disposal: date, ticker, and an occurrence index that
/// distinguishes genuine duplicates (same date AND ticker).
fn keyed_iter(disposals: &[Value]) -> Vec<(&Value, String)> {
    let mut seq: std::collections::HashMap<String, usize> = std::collections::HashMap::new();
    disposals
        .iter()
        .map(|disposal| {
            let base = format!("{}|{}", disposal["date"], disposal["ticker"]);
            let n = seq.entry(base.clone()).or_insert(0);
            let key = format!("{base}|{n}");
            *n += 1;
            (disposal, key)
        })
        .collect()
}

fn key_disposals(disposals: &[Value]) -> std::collections::HashMap<String, &Value> {
    keyed_iter(disposals)
        .into_iter()
        .map(|(disposal, key)| (key, disposal))
        .collect()
}

fn close(fixture: &str, field: &str, new: Decimal, old: Decimal) {
    assert!(
        (new - old).abs() <= MONEY_TOLERANCE,
        "{fixture}: {field}: new {new} differs from golden {old} by more than {MONEY_TOLERANCE}"
    );
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
