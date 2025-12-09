//! PDF formatter for CGT tax reports using embedded Typst.
//!
//! This crate generates professional PDF documents from tax reports
//! without requiring any external tool installation.

use cgt_core::{CgtError, Disposal, MatchRule, Operation, TaxReport, Transaction, get_exemption};
use chrono::{Local, NaiveDate};
use rust_decimal::Decimal;
use typst::foundations::{Dict, IntoValue, Value};
use typst_as_lib::TypstEngine;

// Embed the template at compile time
static TEMPLATE: &str = include_str!("templates/report.typ");

// Embed Roboto font (Apache 2.0 license - see fonts/LICENSE.txt)
static ROBOTO_REGULAR: &[u8] = include_bytes!("../fonts/Roboto-Regular.ttf");
static ROBOTO_BOLD: &[u8] = include_bytes!("../fonts/Roboto-Bold.ttf");

// =============================================================================
// Helper Functions
// =============================================================================

fn format_currency(value: Decimal) -> String {
    let floored = value.floor();
    let abs_value = floored.abs();
    let formatted = format_with_commas(abs_value);
    if floored < Decimal::ZERO {
        format!("-£{formatted}")
    } else {
        format!("£{formatted}")
    }
}

fn format_with_commas(value: Decimal) -> String {
    let s = value.to_string();
    let integer_part = s.split('.').next().unwrap_or("0");

    let chars: Vec<char> = integer_part.chars().collect();
    let mut result = String::with_capacity(chars.len() + chars.len() / 3);
    for (i, c) in chars.iter().enumerate() {
        if i > 0 && (chars.len() - i) % 3 == 0 {
            result.push(',');
        }
        result.push(*c);
    }
    result
}

fn format_decimal(value: Decimal) -> String {
    let s = value.to_string();
    if s.contains('.') {
        s.trim_end_matches('0').trim_end_matches('.').to_string()
    } else {
        s
    }
}

fn format_price(value: Decimal) -> String {
    format!("£{}", format_decimal(value))
}

fn format_date(date: NaiveDate) -> String {
    date.format("%d/%m/%Y").to_string()
}

fn format_tax_year(start_year: u16) -> String {
    format!("{}/{}", start_year, start_year + 1)
}

/// Sort transactions by date, then by ticker for deterministic output.
fn sort_by_date_ticker<T, F, G>(items: &mut [T], get_date: F, get_ticker: G)
where
    F: Fn(&T) -> NaiveDate,
    G: Fn(&T) -> &str,
{
    items.sort_by(|a, b| {
        get_date(a)
            .cmp(&get_date(b))
            .then_with(|| get_ticker(a).cmp(get_ticker(b)))
    });
}

// =============================================================================
// Data Conversion to Typst Dict
// =============================================================================

fn build_template_data(report: &TaxReport, transactions: &[Transaction]) -> Result<Dict, CgtError> {
    let mut data = Dict::new();

    // Generation date
    data.insert(
        "generation_date".into(),
        Local::now().format("%d/%m/%Y").to_string().into_value(),
    );

    // Summary rows
    let summary_rows = build_summary_rows(report)?;
    data.insert("summary_rows".into(), summary_rows.into_value());

    // Tax years with disposals
    let tax_years = build_tax_years(report, transactions);
    data.insert("tax_years".into(), tax_years.into_value());

    // Holdings
    let (has_holdings, holdings_rows) = build_holdings_rows(report);
    data.insert("has_holdings".into(), has_holdings.into_value());
    data.insert("holdings_rows".into(), holdings_rows.into_value());

    // Transactions
    let (has_transactions, transaction_rows) = build_transaction_rows(transactions);
    data.insert("has_transactions".into(), has_transactions.into_value());
    data.insert("transaction_rows".into(), transaction_rows.into_value());

    // Asset events
    let (has_asset_events, asset_event_rows) = build_asset_event_rows(transactions);
    data.insert("has_asset_events".into(), has_asset_events.into_value());
    data.insert("asset_event_rows".into(), asset_event_rows.into_value());

    Ok(data)
}

fn build_summary_rows(report: &TaxReport) -> Result<Vec<Value>, CgtError> {
    let mut rows = Vec::new();
    for year in &report.tax_years {
        let exemption = get_exemption(year.period.start_year())?;
        let proceeds: Decimal = year.disposals.iter().map(|d| d.proceeds).sum();
        let taxable = (year.net_gain - exemption).max(Decimal::ZERO);

        rows.extend([
            format_tax_year(year.period.start_year()).into_value(),
            format_currency(year.net_gain).into_value(),
            format_currency(proceeds).into_value(),
            format_currency(exemption).into_value(),
            format_currency(taxable).into_value(),
        ]);
    }
    Ok(rows)
}

fn build_tax_years(report: &TaxReport, transactions: &[Transaction]) -> Vec<Value> {
    report
        .tax_years
        .iter()
        .map(|year| {
            let mut year_dict = Dict::new();
            year_dict.insert(
                "period".into(),
                format_tax_year(year.period.start_year()).into_value(),
            );

            let mut sorted_disposals: Vec<_> = year.disposals.iter().collect();
            sort_by_date_ticker(&mut sorted_disposals, |d| d.date, |d| &d.ticker);

            let disposals: Vec<Value> = sorted_disposals
                .into_iter()
                .map(|d| build_disposal_dict(d, find_sell_price(d, transactions)).into_value())
                .collect();

            year_dict.insert("disposals".into(), disposals.into_value());
            year_dict.into_value()
        })
        .collect()
}

fn build_holdings_rows(report: &TaxReport) -> (bool, Vec<Value>) {
    let mut active: Vec<_> = report
        .holdings
        .iter()
        .filter(|h| h.quantity > Decimal::ZERO)
        .collect();
    active.sort_by(|a, b| a.ticker.cmp(&b.ticker));

    let rows: Vec<Value> = active
        .iter()
        .flat_map(|h| {
            let cost_basis = (h.total_cost / h.quantity).round_dp(2);
            [
                h.ticker.clone().into_value(),
                format_decimal(h.quantity).into_value(),
                format_price(cost_basis).into_value(),
            ]
        })
        .collect();

    (!active.is_empty(), rows)
}

fn build_transaction_rows(transactions: &[Transaction]) -> (bool, Vec<Value>) {
    let mut txns: Vec<_> = transactions
        .iter()
        .filter_map(|t| match &t.operation {
            Operation::Buy {
                amount,
                price,
                expenses,
            } => Some((t.date, &t.ticker, "BUY", *amount, *price, *expenses)),
            Operation::Sell {
                amount,
                price,
                expenses,
            } => Some((t.date, &t.ticker, "SELL", *amount, *price, *expenses)),
            _ => None,
        })
        .collect();

    txns.sort_by(|a, b| a.0.cmp(&b.0).then_with(|| a.1.cmp(b.1)));

    let rows: Vec<Value> = txns
        .into_iter()
        .flat_map(|(date, ticker, op_type, amount, price, expenses)| {
            [
                format_date(date).into_value(),
                op_type.into_value(),
                ticker.clone().into_value(),
                format_decimal(amount).into_value(),
                format_price(price).into_value(),
                format_price(expenses).into_value(),
            ]
        })
        .collect();

    (!rows.is_empty(), rows)
}

fn build_asset_event_rows(transactions: &[Transaction]) -> (bool, Vec<Value>) {
    let mut events: Vec<_> = transactions
        .iter()
        .filter_map(|t| {
            let (op_type, amount, value) = match &t.operation {
                Operation::Dividend {
                    amount,
                    total_value,
                    ..
                } => (
                    "DIVIDEND",
                    format_decimal(*amount),
                    format_currency(*total_value),
                ),
                Operation::CapReturn {
                    amount,
                    total_value,
                    ..
                } => (
                    "CAPRETURN",
                    format_decimal(*amount),
                    format_currency(*total_value),
                ),
                Operation::Split { ratio } => ("SPLIT", format_decimal(*ratio), "-".to_string()),
                Operation::Unsplit { ratio } => {
                    ("UNSPLIT", format_decimal(*ratio), "-".to_string())
                }
                _ => return None,
            };
            Some((t.date, &t.ticker, op_type, amount, value))
        })
        .collect();

    events.sort_by(|a, b| a.0.cmp(&b.0).then_with(|| a.1.cmp(b.1)));

    let rows: Vec<Value> = events
        .into_iter()
        .flat_map(|(date, ticker, op_type, amount, value)| {
            [
                format_date(date).into_value(),
                op_type.into_value(),
                ticker.clone().into_value(),
                amount.into_value(),
                value.into_value(),
            ]
        })
        .collect();

    (!rows.is_empty(), rows)
}

fn find_sell_price(disposal: &Disposal, transactions: &[Transaction]) -> Decimal {
    transactions
        .iter()
        .find_map(|t| {
            if t.ticker == disposal.ticker
                && t.date == disposal.date
                && let Operation::Sell { price, .. } = &t.operation
            {
                return Some(*price);
            }
            None
        })
        .unwrap_or_else(|| {
            if disposal.quantity != Decimal::ZERO {
                disposal.proceeds / disposal.quantity
            } else {
                Decimal::ZERO
            }
        })
}

fn build_disposal_dict(disposal: &Disposal, sell_price: Decimal) -> Dict {
    let mut dict = Dict::new();

    let total_gain: Decimal = disposal.matches.iter().map(|m| m.gain_or_loss).sum();
    let gain_type = if total_gain >= Decimal::ZERO {
        "GAIN"
    } else {
        "LOSS"
    };

    dict.insert("ticker".into(), disposal.ticker.clone().into_value());
    dict.insert("date".into(), format_date(disposal.date).into_value());
    dict.insert(
        "quantity".into(),
        format_decimal(disposal.quantity).into_value(),
    );
    dict.insert("gain_type".into(), gain_type.into_value());
    dict.insert(
        "gain_amount".into(),
        format_currency(total_gain.abs()).into_value(),
    );

    // Match descriptions
    let matches: Vec<Value> = disposal
        .matches
        .iter()
        .map(|m| {
            let description = format_match_description(m);
            let mut match_dict = Dict::new();
            match_dict.insert("description".into(), description.into_value());
            match_dict.into_value()
        })
        .collect();
    dict.insert("matches".into(), matches.into_value());

    // Proceeds calculation
    dict.insert(
        "proceeds_calc".into(),
        format!(
            "{} × {} = {}",
            format_decimal(disposal.quantity),
            format_price(sell_price),
            format_currency(disposal.proceeds)
        )
        .into_value(),
    );

    let total_cost: Decimal = disposal.matches.iter().map(|m| m.allowable_cost).sum();
    dict.insert(
        "total_cost".into(),
        format_currency(total_cost).into_value(),
    );
    dict.insert("result".into(), format_currency(total_gain).into_value());

    dict
}

fn format_match_description(m: &cgt_core::Match) -> String {
    match m.rule {
        MatchRule::SameDay => format!("Same Day: {} shares", format_decimal(m.quantity)),
        MatchRule::BedAndBreakfast => {
            let qty = format_decimal(m.quantity);
            match m.acquisition_date {
                Some(date) => format!("B&B: {qty} shares from {}", format_date(date)),
                None => format!("B&B: {qty} shares"),
            }
        }
        MatchRule::Section104 => {
            let cost_per_share = if m.quantity != Decimal::ZERO {
                (m.allowable_cost / m.quantity).round_dp(2)
            } else {
                Decimal::ZERO
            };
            format!(
                "Section 104: {} shares @ {}",
                format_decimal(m.quantity),
                format_price(cost_per_share)
            )
        }
    }
}

// =============================================================================
// Main Format Function
// =============================================================================

/// Generate a PDF report from tax data.
///
/// # Arguments
/// * `report` - The calculated tax report
/// * `transactions` - Original transactions for display
///
/// # Returns
/// PDF file contents as bytes, or error if generation fails.
///
/// # Errors
/// Returns `CgtError::PdfGeneration` if Typst compilation or PDF export fails.
/// Returns `CgtError::UnsupportedExemptionYear` if a tax year's exemption is unavailable.
pub fn format(report: &TaxReport, transactions: &[Transaction]) -> Result<Vec<u8>, CgtError> {
    let data = build_template_data(report, transactions)?;

    let engine = TypstEngine::builder()
        .main_file(TEMPLATE)
        .fonts([ROBOTO_REGULAR, ROBOTO_BOLD])
        .build();

    let compiled = engine.compile_with_input(data);
    let doc = compiled
        .output
        .map_err(|e| CgtError::PdfGeneration(format!("Typst compilation failed: {e}")))?;

    let pdf = typst_pdf::pdf(&doc, &typst_pdf::PdfOptions::default())
        .map_err(|e| CgtError::PdfGeneration(format!("PDF export failed: {e:?}")))?;

    Ok(pdf)
}

#[cfg(test)]
#[allow(clippy::expect_used)]
mod tests {
    use super::*;

    #[test]
    fn test_format_currency() {
        assert_eq!(format_currency(Decimal::from(100)), "£100");
        assert_eq!(format_currency(Decimal::from(-20)), "-£20");
        assert_eq!(format_currency(Decimal::from(1234)), "£1,234");
        assert_eq!(format_currency(Decimal::from(1000000)), "£1,000,000");
    }

    #[test]
    fn test_format_date() {
        let date = NaiveDate::from_ymd_opt(2018, 8, 28).expect("valid date");
        assert_eq!(format_date(date), "28/08/2018");
    }

    #[test]
    fn test_pdf_starts_with_header() {
        use cgt_core::{TaxPeriod, TaxYearSummary};

        let report = TaxReport {
            tax_years: vec![TaxYearSummary {
                period: TaxPeriod::new(2023).expect("valid year"),
                disposals: vec![],
                total_gain: Decimal::ZERO,
                total_loss: Decimal::ZERO,
                net_gain: Decimal::ZERO,
            }],
            holdings: vec![],
        };

        let pdf = format(&report, &[]).expect("PDF generation should succeed");
        assert!(
            pdf.starts_with(b"%PDF"),
            "PDF should start with %PDF header"
        );
    }
}
