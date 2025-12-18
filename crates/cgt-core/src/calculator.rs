use crate::error::CgtError;
use crate::matcher::{MatchResult, Matcher};
use crate::models::*;
use cgt_money::FxCache;
use chrono::NaiveDate;
use rust_decimal::Decimal;
use std::collections::HashMap;

/// Calculate CGT report.
///
/// If `tax_year_start` is `Some(year)`, only disposals in that tax year are included.
/// If `tax_year_start` is `None`, all tax years with disposals are included.
///
/// # Validation
///
/// This function does not perform input validation. Callers should use
/// [`crate::validation::validate()`] before calling this function to catch
/// invalid inputs (zero quantities, negative prices, etc.) with helpful error
/// messages. Invalid inputs may cause unexpected behavior or incorrect results.
///
/// ```ignore
/// use cgt_core::{validation, calculator};
///
/// let result = validation::validate(&transactions);
/// if !result.is_valid() {
///     // Handle validation errors
/// }
/// let report = calculator::calculate(transactions, None, None)?;
/// ```
pub fn calculate(
    transactions: Vec<Transaction>,
    tax_year_start: Option<i32>,
    fx_cache: Option<&FxCache>,
) -> Result<TaxReport, CgtError> {
    // Convert all transactions to GBP-normalized form
    let transactions = transactions_to_gbp(&transactions, fx_cache)?;

    // Use the Matcher module for all share matching logic
    let mut matcher = Matcher::new();
    let (match_results, pools) = matcher.process(transactions)?;

    // Build tax year summaries
    let tax_years = match tax_year_start {
        Some(year) => {
            // Filter matches for the requested tax year
            let summary = build_tax_year_summary(year, &match_results)?;
            vec![summary]
        }
        None => {
            // Group all matches by tax year
            build_all_tax_year_summaries(&match_results)?
        }
    };

    // Convert pools to sorted Vec for output
    let mut holdings: Vec<Section104Holding> = pools.into_values().collect();
    holdings.sort_by(|a, b| a.ticker.cmp(&b.ticker));

    Ok(TaxReport {
        tax_years,
        holdings,
    })
}

/// Build a summary for a single tax year.
fn build_tax_year_summary(
    tax_year_start: i32,
    match_results: &[MatchResult],
) -> Result<TaxYearSummary, CgtError> {
    let start_date =
        chrono::NaiveDate::from_ymd_opt(tax_year_start, 4, 6).ok_or(CgtError::InvalidDateYear {
            year: tax_year_start,
        })?;
    let end_date = chrono::NaiveDate::from_ymd_opt(tax_year_start + 1, 4, 5).ok_or(
        CgtError::InvalidDateYear {
            year: tax_year_start + 1,
        },
    )?;

    let year_matches: Vec<MatchResult> = match_results
        .iter()
        .filter(|m| m.disposal_date >= start_date && m.disposal_date <= end_date)
        .cloned()
        .collect();

    let disposals = group_matches_into_disposals(year_matches);
    let (total_gain, total_loss) = calculate_totals(&disposals);
    let tax_period = TaxPeriod::from_date(start_date);

    Ok(TaxYearSummary {
        period: tax_period,
        disposals,
        total_gain,
        total_loss,
        net_gain: total_gain - total_loss,
    })
}

/// Build summaries for all tax years that have disposals.
fn build_all_tax_year_summaries(
    match_results: &[MatchResult],
) -> Result<Vec<TaxYearSummary>, CgtError> {
    // Group matches by tax year
    let mut matches_by_year: HashMap<u16, Vec<MatchResult>> = HashMap::new();

    for m in match_results {
        let tax_period = TaxPeriod::from_date(m.disposal_date);
        matches_by_year
            .entry(tax_period.start_year())
            .or_default()
            .push(m.clone());
    }

    // Build summaries for each year
    let mut summaries: Vec<TaxYearSummary> = Vec::new();

    for (year, year_matches) in matches_by_year {
        let tax_period =
            TaxPeriod::new(year).map_err(|_| CgtError::InvalidDateYear { year: year as i32 })?;
        let disposals = group_matches_into_disposals(year_matches);
        let (total_gain, total_loss) = calculate_totals(&disposals);

        summaries.push(TaxYearSummary {
            period: tax_period,
            disposals,
            total_gain,
            total_loss,
            net_gain: total_gain - total_loss,
        });
    }

    // Sort by tax year (chronological order)
    summaries.sort_by_key(|s| s.period.start_year());

    Ok(summaries)
}

/// Calculate total gains and losses from disposals.
fn calculate_totals(disposals: &[Disposal]) -> (Decimal, Decimal) {
    let total_gain: Decimal = disposals
        .iter()
        .flat_map(|d| &d.matches)
        .map(|m| {
            if m.gain_or_loss > Decimal::ZERO {
                m.gain_or_loss
            } else {
                Decimal::ZERO
            }
        })
        .sum();

    let total_loss: Decimal = disposals
        .iter()
        .flat_map(|d| &d.matches)
        .map(|m| {
            if m.gain_or_loss < Decimal::ZERO {
                m.gain_or_loss.abs()
            } else {
                Decimal::ZERO
            }
        })
        .sum();

    (total_gain, total_loss)
}

/// Group match results into Disposal objects by (date, ticker)
fn group_matches_into_disposals(match_results: Vec<MatchResult>) -> Vec<Disposal> {
    // Group by (date, ticker)
    let mut disposal_map: HashMap<(NaiveDate, String), Vec<MatchResult>> = HashMap::new();

    for m in match_results {
        let key = (m.disposal_date, m.disposal_ticker.clone());
        disposal_map.entry(key).or_default().push(m);
    }

    // Convert to Disposal structs
    let mut disposals: Vec<Disposal> = disposal_map
        .into_iter()
        .map(|((date, ticker), matches)| {
            // Round to avoid tiny precision errors from proportional fee allocation
            let total_proceeds: Decimal = matches
                .iter()
                .map(|m| m.proceeds)
                .sum::<Decimal>()
                .round_dp(10);
            let total_quantity: Decimal = matches.iter().map(|m| m.quantity).sum();

            let converted_matches: Vec<Match> = matches
                .into_iter()
                .map(|m| Match {
                    rule: m.rule,
                    quantity: m.quantity,
                    allowable_cost: m.allowable_cost,
                    gain_or_loss: m.gain_or_loss,
                    acquisition_date: m.acquisition_date,
                })
                .collect();

            Disposal {
                date,
                ticker,
                quantity: total_quantity,
                proceeds: total_proceeds,
                matches: converted_matches,
            }
        })
        .collect();

    // Sort disposals by date for consistent output
    disposals.sort_by(|a, b| a.date.cmp(&b.date));

    disposals
}
