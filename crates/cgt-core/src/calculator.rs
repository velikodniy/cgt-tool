use crate::error::CgtError;
use crate::matcher::{MatchResult, Matcher};
use crate::models::*;
use cgt_money::FxCache;
use chrono::NaiveDate;
use rust_decimal::Decimal;
use std::collections::HashMap;

pub fn calculate(
    transactions: Vec<Transaction>,
    tax_year_start: i32,
    fx_cache: Option<&FxCache>,
) -> Result<TaxReport, CgtError> {
    // Convert all transactions to GBP-normalized form
    let transactions = transactions_to_gbp(&transactions, fx_cache)?;

    // Use the Matcher module for all share matching logic
    let mut matcher = Matcher::new();
    let (match_results, pools) = matcher.process(transactions)?;

    // Filter matches for the requested tax year
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
        .into_iter()
        .filter(|m| m.disposal_date >= start_date && m.disposal_date <= end_date)
        .collect();

    // Group matches into disposals by (date, ticker)
    let disposals = group_matches_into_disposals(year_matches);

    // Calculate totals
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

    // Create tax year summary
    let tax_period = TaxPeriod::from_date(start_date);
    let tax_year_summary = TaxYearSummary {
        period: tax_period,
        disposals,
        total_gain,
        total_loss,
        net_gain: total_gain - total_loss,
    };

    // Convert pools to sorted Vec for output
    let mut holdings: Vec<Section104Holding> = pools.into_values().collect();
    holdings.sort_by(|a, b| a.ticker.cmp(&b.ticker));

    Ok(TaxReport {
        tax_years: vec![tax_year_summary],
        holdings,
    })
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
