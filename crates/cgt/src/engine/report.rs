//! Report assembly: fold the priced disposals, dividend events, and config
//! exemptions into the public [`crate::report::TaxReport`].
//!
//! Grouping, bucketing, and per-disposal rounding shapes are load-bearing for
//! output equivalence; rust_decimal is path-dependent.

use std::collections::BTreeMap;

use rust_decimal::Decimal;

use super::normalize::{EventKind, EventStream};
use super::plan::MatchPlan;
use super::value::{LegRule, PricedDisposal, ValuedReport};
use crate::config::Config;
use crate::error::CgtError;
use crate::format::round_money;
use crate::model::{TaxPeriod, Transaction};
use crate::report::{Disposal, Holding, MatchLeg, MatchRule, TaxReport, TaxYearSummary};

/// Build the public report from the Phase 2 valuation, the match plan, the
/// dividend events, and the config exemptions.
///
/// `tax_year_start` filters to a single tax year when `Some`; when `None`,
/// disposals group by their own tax year. The exemption lookup is fallible
/// and propagates through `?`.
pub(crate) fn build(
    stream: &EventStream,
    valued: &ValuedReport,
    plan: &MatchPlan,
    tax_year_start: Option<i32>,
    config: &Config,
    transactions: Option<Vec<Transaction>>,
) -> Result<TaxReport, CgtError> {
    // The plan and the valuation are both produced by walking the stream in
    // order, so their disposal vectors align by index. The plan's
    // `matched_quantity` is the merged sell quantity reported per disposal.
    let disposals: Vec<Disposal> = valued
        .disposals
        .iter()
        .zip(plan.disposals.iter())
        .map(|(priced, planned)| map_disposal(priced, planned.matched_quantity()))
        .collect();

    let dividends = aggregate_dividends(stream)?;

    let mut warnings = Vec::new();
    let tax_years = match tax_year_start {
        Some(year) => build_single_year(year, &disposals, &dividends, config, &mut warnings)?,
        None => build_all_years(disposals, &dividends, config, &mut warnings)?,
    };

    let holdings = valued
        .holdings
        .iter()
        .map(|holding| Holding {
            ticker: holding.ticker.clone(),
            quantity: holding.quantity,
            total_cost: holding.total_cost,
        })
        .collect();

    Ok(TaxReport {
        tax_years,
        holdings,
        transactions,
        warnings,
    })
}

/// Map a priced disposal to the public model. Per-leg allowable cost and gain
/// are rounded to 2dp here (the single money policy) so bucketing and the year
/// totals derive from the same rounded figures every published identity foots
/// against — `net_gain == total_gain - total_loss`, and year totals equal the
/// sum of the displayed rows (HMRC SA108 rounds per disposal, then sums).
/// Per-disposal proceeds sums stay at 10dp on the model so the unit-price
/// display divides exact figures.
fn map_disposal(priced: &PricedDisposal, quantity: Decimal) -> Disposal {
    let mut gross_proceeds = Decimal::ZERO;
    let mut proceeds = Decimal::ZERO;
    let mut legs = Vec::with_capacity(priced.legs.len());
    for leg in &priced.legs {
        gross_proceeds += leg.gross_proceeds;
        proceeds += leg.net_proceeds;
        legs.push(MatchLeg {
            rule: map_rule(leg.rule),
            quantity: leg.quantity,
            allowable_cost: round_money(leg.allowable_cost),
            gain_or_loss: round_money(leg.gain_or_loss),
            acquisition_date: leg.acquisition_date,
        });
    }
    Disposal {
        date: priced.date,
        ticker: priced.ticker.clone(),
        quantity,
        gross_proceeds: gross_proceeds.round_dp(10),
        proceeds: proceeds.round_dp(10),
        legs,
    }
}

fn map_rule(rule: LegRule) -> MatchRule {
    match rule {
        LegRule::SameDay => MatchRule::SameDay,
        LegRule::BedAndBreakfast => MatchRule::BedAndBreakfast,
        LegRule::Section104 => MatchRule::Section104,
    }
}

/// Dividend income and tax paid, GBP, keyed by tax-year start year.
struct DividendTotals {
    income: Decimal,
    tax_paid: Decimal,
}

/// Aggregate DIVIDEND events by tax year. Values are already GBP-converted.
fn aggregate_dividends(stream: &EventStream) -> Result<BTreeMap<u16, DividendTotals>, CgtError> {
    let mut totals: BTreeMap<u16, DividendTotals> = BTreeMap::new();
    for event in stream.events() {
        if let EventKind::Dividend {
            total_value,
            tax_paid,
        } = &event.kind
        {
            let period = TaxPeriod::from_date(event.date)?;
            let entry = totals.entry(period.start_year()).or_insert(DividendTotals {
                income: Decimal::ZERO,
                tax_paid: Decimal::ZERO,
            });
            entry.income += *total_value;
            entry.tax_paid += *tax_paid;
        }
    }
    Ok(totals)
}

/// Filter all disposals into a single tax year `[Apr 6 .. Apr 5]`.
fn build_single_year(
    year: i32,
    disposals: &[Disposal],
    dividends: &BTreeMap<u16, DividendTotals>,
    config: &Config,
    warnings: &mut Vec<String>,
) -> Result<Vec<TaxYearSummary>, CgtError> {
    let start_year = u16::try_from(year).map_err(|_| CgtError::InvalidDateYear { year })?;
    let period = TaxPeriod::new(start_year)?;
    let (Some(start), Some(end)) = (period.start_date(), period.end_date()) else {
        return Err(CgtError::InvalidDateYear { year });
    };
    let in_year: Vec<Disposal> = disposals
        .iter()
        .filter(|disposal| disposal.date >= start && disposal.date <= end)
        .cloned()
        .collect();
    let summary = summarize(period, in_year, dividends, config, warnings)?;
    Ok(vec![summary])
}

/// Group disposals by their own tax year, sorted by start year. Years with
/// dividends but no disposals are not emitted.
fn build_all_years(
    disposals: Vec<Disposal>,
    dividends: &BTreeMap<u16, DividendTotals>,
    config: &Config,
    warnings: &mut Vec<String>,
) -> Result<Vec<TaxYearSummary>, CgtError> {
    let mut grouped: BTreeMap<u16, Vec<Disposal>> = BTreeMap::new();
    for disposal in disposals {
        let period = TaxPeriod::from_date(disposal.date)?;
        grouped
            .entry(period.start_year())
            .or_default()
            .push(disposal);
    }
    let mut summaries = Vec::with_capacity(grouped.len());
    for (start_year, year_disposals) in grouped {
        let period = TaxPeriod::new(start_year)?;
        summaries.push(summarize(
            period,
            year_disposals,
            dividends,
            config,
            warnings,
        )?);
    }
    Ok(summaries)
}

/// Build one year's summary from its disposals and the dividend lookup.
fn summarize(
    period: TaxPeriod,
    mut disposals: Vec<Disposal>,
    dividends: &BTreeMap<u16, DividendTotals>,
    config: &Config,
    warnings: &mut Vec<String>,
) -> Result<TaxYearSummary, CgtError> {
    // Stable sort by (date, ticker): the input order of equal keys is kept.
    disposals.sort_by(|a, b| a.date.cmp(&b.date).then_with(|| a.ticker.cmp(&b.ticker)));

    let mut total_gain = Decimal::ZERO;
    let mut total_loss = Decimal::ZERO;
    let mut gross_proceeds = Decimal::ZERO;
    let mut total_allowable_cost = Decimal::ZERO;
    for disposal in &disposals {
        // Legs carry 2dp-rounded gains and costs (map_disposal), so the net and
        // the totals are exact at 2dp and cannot be flipped by sub-penny fee
        // dust. Bucketing thresholds are load-bearing: a per-disposal net of
        // exactly zero contributes to neither the gain nor the loss total.
        let net: Decimal = disposal.legs.iter().map(|leg| leg.gain_or_loss).sum();
        if net > Decimal::ZERO {
            total_gain += net;
        } else if net < Decimal::ZERO {
            total_loss += net.abs();
        }
        // Sum the rounded per-disposal proceeds so the year total foots against
        // the displayed disposal rows.
        gross_proceeds += round_money(disposal.gross_proceeds);
        total_allowable_cost += disposal
            .legs
            .iter()
            .map(|leg| leg.allowable_cost)
            .sum::<Decimal>();
    }

    let net_gain = total_gain - total_loss;
    let exempt_amount = match config.get_exemption(period.start_year()) {
        Ok(amount) => amount,
        Err(_) if config.allow_missing_exemption => {
            warnings.push(format!(
                "{period}: no configured exemption; applying no allowance"
            ));
            Decimal::ZERO
        }
        Err(e) => return Err(e),
    };
    let taxable_gain = (net_gain - exempt_amount).max(Decimal::ZERO);

    let dividend = dividends.get(&period.start_year());
    let dividend_income = dividend.map_or(Decimal::ZERO, |d| d.income);
    let dividend_tax_paid = dividend.map_or(Decimal::ZERO, |d| d.tax_paid);

    let disposal_count = disposals.len();
    Ok(TaxYearSummary {
        period,
        disposals,
        disposal_count,
        total_gain,
        total_loss,
        net_gain,
        gross_proceeds,
        total_allowable_cost,
        exempt_amount,
        taxable_gain,
        dividend_income,
        dividend_tax_paid,
    })
}

#[cfg(test)]
mod tests {
    use rust_decimal_macros::dec;

    use super::build;
    use crate::config::Config;
    use crate::engine::normalize::normalize;
    use crate::engine::plan::plan;
    use crate::engine::value::value;
    use crate::report::MatchRule;

    fn report(input: &str, tax_year_start: Option<i32>) -> crate::report::TaxReport {
        let transactions = crate::dsl::parse(input).expect("test DSL parses");
        let stream = normalize(&transactions, None).expect("test input normalizes");
        let match_plan = plan(&stream).expect("test input plans");
        let valued = value(&stream, &match_plan).expect("test input values");
        let config = Config::embedded().expect("embedded config loads");
        build(&stream, &valued, &match_plan, tax_year_start, &config, None).expect("report builds")
    }

    #[test]
    fn single_disposal_reports_one_leg_and_a_holding() {
        let report = report(
            "2023-01-01 BUY ABC 10 @ 10.00 GBP FEES 5.00 GBP\n\
             2023-06-01 SELL ABC 4 @ 20.00 GBP FEES 2.00 GBP\n",
            None,
        );
        assert_eq!(report.tax_years.len(), 1);
        let year = &report.tax_years[0];
        // 2023-06-01 is on or after 6 April, so the tax year is 2023/24.
        assert_eq!(year.period.start_year(), 2023);
        assert_eq!(year.disposal_count, 1);
        let disposal = &year.disposals[0];
        assert_eq!(disposal.quantity, dec!(4));
        assert_eq!(disposal.legs.len(), 1);
        assert_eq!(disposal.legs[0].rule, MatchRule::Section104);
        // Net 78, cost 42, gain 36.
        assert_eq!(year.total_gain, dec!(36));
        assert_eq!(year.total_loss, dec!(0));
        assert_eq!(year.net_gain, dec!(36));

        let abc = report
            .holdings
            .iter()
            .find(|holding| holding.ticker == "ABC")
            .expect("ABC holding present");
        assert_eq!(abc.quantity, dec!(6));
        assert_eq!(abc.total_cost, dec!(63));
    }

    #[test]
    fn quantity_comes_from_the_merged_match_plan() {
        // Same-day buy plus a pool match: the reported quantity is the full
        // merged sell, sourced from the plan's matched quantity.
        let report = report(
            "2023-01-01 BUY ABC 10 @ 10.00 GBP\n\
             2023-06-01 BUY ABC 5 @ 12.00 GBP\n\
             2023-06-01 SELL ABC 8 @ 13.00 GBP\n",
            None,
        );
        let disposal = &report.tax_years[0].disposals[0];
        assert_eq!(disposal.quantity, dec!(8));
        assert_eq!(disposal.legs.len(), 2);
    }

    #[test]
    fn zero_quantity_holdings_are_included() {
        let report = report(
            "2023-01-01 BUY ABC 10 @ 10.00 GBP\n\
             2023-06-01 SELL ABC 10 @ 12.00 GBP\n",
            None,
        );
        let abc = report
            .holdings
            .iter()
            .find(|holding| holding.ticker == "ABC")
            .expect("drained ABC holding present");
        assert_eq!(abc.quantity, dec!(0));
        assert_eq!(abc.total_cost, dec!(0));
    }

    #[test]
    fn dividend_only_year_is_dropped() {
        // The dividend year (2023/24) has no disposal, so no summary is emitted
        // for it; the disposal year carries its own dividend total only.
        let report = report(
            "2023-06-01 DIVIDEND ABC TOTAL 50.00 GBP TAX 5.00 GBP\n\
             2024-06-01 BUY ABC 10 @ 10.00 GBP\n\
             2024-12-01 SELL ABC 10 @ 12.00 GBP\n",
            None,
        );
        let years: Vec<u16> = report
            .tax_years
            .iter()
            .map(|year| year.period.start_year())
            .collect();
        // 2023 (dividend-only) is absent; only 2024 (the disposal year) shows.
        assert_eq!(years, vec![2024]);
        assert_eq!(report.tax_years[0].dividend_income, dec!(0));
    }

    #[test]
    fn dividend_in_a_disposal_year_is_aggregated() {
        let report = report(
            "2023-06-01 DIVIDEND ABC TOTAL 50.00 GBP TAX 5.00 GBP\n\
             2023-07-01 BUY ABC 10 @ 10.00 GBP\n\
             2023-12-01 SELL ABC 10 @ 12.00 GBP\n",
            None,
        );
        assert_eq!(report.tax_years.len(), 1);
        let year = &report.tax_years[0];
        assert_eq!(year.dividend_income, dec!(50));
        assert_eq!(year.dividend_tax_paid, dec!(5));
    }

    #[test]
    fn single_year_filter_keeps_only_in_range_disposals() {
        let report = report(
            "2022-06-01 BUY ABC 10 @ 10.00 GBP\n\
             2022-12-01 SELL ABC 5 @ 12.00 GBP\n\
             2023-12-01 SELL ABC 5 @ 12.00 GBP\n",
            Some(2022),
        );
        assert_eq!(report.tax_years.len(), 1);
        let year = &report.tax_years[0];
        assert_eq!(year.period.start_year(), 2022);
        // Only the 2022-12-01 disposal is inside [2022-04-06 .. 2023-04-05].
        assert_eq!(year.disposal_count, 1);
        assert_eq!(year.disposals[0].date.to_string(), "2022-12-01");
    }

    #[test]
    fn exempt_amount_reduces_taxable_gain_to_zero_floor() {
        // A small gain is fully covered by the exemption; taxable gain floors.
        let report = report(
            "2023-01-01 BUY ABC 10 @ 10.00 GBP\n\
             2023-06-01 SELL ABC 10 @ 11.00 GBP\n",
            None,
        );
        let year = &report.tax_years[0];
        assert!(year.exempt_amount > dec!(0));
        assert_eq!(year.taxable_gain, dec!(0));
    }
}
