//! PDF formatter for CGT tax reports using embedded Typst.
//!
//! This crate generates professional PDF documents from tax reports
//! without requiring any external tool installation.

use cgt_core::{
    CgtError, CurrencyAmount, Disposal, MatchRule, Operation, TaxReport, Transaction, get_exemption,
};
use chrono::{Datelike, Local, NaiveDate};
use rust_decimal::Decimal;
use rust_decimal::prelude::ToPrimitive;
use thiserror::Error;
use typst::foundations::{Dict, IntoValue, Value};
use typst_as_lib::TypstEngine;

// Embed the template at compile time
static TEMPLATE: &str = include_str!("templates/report.typ");

// Embed Roboto font (Apache 2.0 license - see fonts/LICENSE.txt)
static ROBOTO_REGULAR: &[u8] = include_bytes!("../fonts/Roboto-Regular.ttf");
static ROBOTO_BOLD: &[u8] = include_bytes!("../fonts/Roboto-Bold.ttf");

#[derive(Debug, Error)]
pub enum PdfError {
    #[error(transparent)]
    Core(#[from] CgtError),

    #[error("PDF generation failed: failed to convert Decimal to float")]
    DecimalToFloat,

    #[error("PDF generation failed: Typst compilation failed: {0}")]
    TypstCompilation(String),

    #[error("PDF generation failed: PDF export failed: {0}")]
    PdfExport(String),
}

fn decimal_to_f64(value: Decimal) -> Result<f64, PdfError> {
    value.to_f64().ok_or(PdfError::DecimalToFloat)
}

fn decimal_to_value(value: Decimal) -> Result<Value, PdfError> {
    Ok(decimal_to_f64(value)?.into_value())
}

fn date_dict(date: NaiveDate) -> Dict {
    let mut dict = Dict::new();
    dict.insert("year".into(), (date.year() as i64).into_value());
    dict.insert("month".into(), (date.month() as i64).into_value());
    dict.insert("day".into(), (date.day() as i64).into_value());
    dict
}

fn optional_date_value(date: Option<NaiveDate>) -> Value {
    match date {
        Some(value) => date_dict(value).into_value(),
        None => Value::None,
    }
}

fn currency_amount_value(amount: &CurrencyAmount) -> Result<Value, PdfError> {
    let mut dict = Dict::new();
    dict.insert("amount".into(), decimal_to_value(amount.amount)?);
    dict.insert(
        "currency".into(),
        amount.currency.code().to_string().into_value(),
    );
    Ok(dict.into_value())
}

fn match_rule_label(rule: &MatchRule) -> &'static str {
    match rule {
        MatchRule::SameDay => "SAME_DAY",
        MatchRule::BedAndBreakfast => "BED_AND_BREAKFAST",
        MatchRule::Section104 => "SECTION_104",
    }
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

fn build_template_data(report: &TaxReport, transactions: &[Transaction]) -> Result<Dict, PdfError> {
    let mut data = Dict::new();

    // Generation date
    let today = Local::now().date_naive();
    data.insert("generation_date".into(), date_dict(today).into_value());

    // Summary rows
    let summary_rows = build_summary_rows(report)?;
    data.insert("summary_rows".into(), summary_rows.into_value());

    // Tax years with disposals
    let tax_years = build_tax_years(report)?;
    data.insert("tax_years".into(), tax_years.into_value());

    // Holdings
    let (has_holdings, holdings_rows) = build_holdings_rows(report)?;
    data.insert("has_holdings".into(), has_holdings.into_value());
    data.insert("holdings_rows".into(), holdings_rows.into_value());

    // Transactions
    let (has_transactions, transaction_rows) = build_transaction_rows(transactions)?;
    data.insert("has_transactions".into(), has_transactions.into_value());
    data.insert("transaction_rows".into(), transaction_rows.into_value());

    // Asset events
    let (has_asset_events, asset_event_rows) = build_asset_event_rows(transactions)?;
    data.insert("has_asset_events".into(), has_asset_events.into_value());
    data.insert("asset_event_rows".into(), asset_event_rows.into_value());

    Ok(data)
}

fn build_summary_rows(report: &TaxReport) -> Result<Vec<Value>, PdfError> {
    let mut rows = Vec::new();
    for year in &report.tax_years {
        let exemption = get_exemption(year.period.start_year())?;
        let gross_proceeds = year.gross_proceeds();
        let taxable = year.taxable_gain(exemption);

        let mut row = Dict::new();
        row.insert(
            "start_year".into(),
            (year.period.start_year() as i64).into_value(),
        );
        row.insert(
            "disposal_count".into(),
            (year.disposal_count() as i64).into_value(),
        );
        row.insert("net_gain".into(), decimal_to_value(year.net_gain)?);
        row.insert("total_gain".into(), decimal_to_value(year.total_gain)?);
        row.insert("total_loss".into(), decimal_to_value(year.total_loss)?);
        row.insert("gross_proceeds".into(), decimal_to_value(gross_proceeds)?);
        row.insert("exemption".into(), decimal_to_value(exemption)?);
        row.insert("taxable".into(), decimal_to_value(taxable)?);
        rows.push(row.into_value());
    }
    Ok(rows)
}

fn build_tax_years(report: &TaxReport) -> Result<Vec<Value>, PdfError> {
    report
        .tax_years
        .iter()
        .map(|year| {
            let mut year_dict = Dict::new();
            year_dict.insert(
                "start_year".into(),
                (year.period.start_year() as i64).into_value(),
            );

            let mut sorted_disposals: Vec<_> = year.disposals.iter().collect();
            sort_by_date_ticker(&mut sorted_disposals, |d| d.date, |d| &d.ticker);

            let disposals: Vec<Value> = sorted_disposals
                .into_iter()
                .map(build_disposal_dict)
                .map(|result| result.map(IntoValue::into_value))
                .collect::<Result<Vec<_>, PdfError>>()?;

            year_dict.insert("disposals".into(), disposals.into_value());
            Ok(year_dict.into_value())
        })
        .collect()
}

fn build_holdings_rows(report: &TaxReport) -> Result<(bool, Vec<Value>), PdfError> {
    let mut active: Vec<_> = report
        .holdings
        .iter()
        .filter(|h| h.quantity > Decimal::ZERO)
        .collect();
    active.sort_by(|a, b| a.ticker.cmp(&b.ticker));

    let rows: Vec<Value> = active
        .iter()
        .map(|h| {
            let mut row = Dict::new();
            row.insert("ticker".into(), h.ticker.clone().into_value());
            row.insert("quantity".into(), decimal_to_value(h.quantity)?);
            row.insert("total_cost".into(), decimal_to_value(h.total_cost)?);
            Ok(row.into_value())
        })
        .collect::<Result<Vec<_>, PdfError>>()?;

    Ok((!active.is_empty(), rows))
}

fn build_transaction_rows(transactions: &[Transaction]) -> Result<(bool, Vec<Value>), PdfError> {
    let mut rows: Vec<(NaiveDate, String, Dict)> = Vec::new();

    for transaction in transactions {
        let (op_type, amount, price, fees) = match &transaction.operation {
            Operation::Buy {
                amount,
                price,
                fees,
            } => ("BUY", *amount, price, fees),
            Operation::Sell {
                amount,
                price,
                fees,
            } => ("SELL", *amount, price, fees),
            _ => continue,
        };

        let mut row = Dict::new();
        row.insert("date".into(), date_dict(transaction.date).into_value());
        row.insert("type".into(), op_type.into_value());
        row.insert("ticker".into(), transaction.ticker.clone().into_value());
        row.insert("quantity".into(), decimal_to_value(amount)?);
        row.insert("price".into(), currency_amount_value(price)?);
        row.insert("fees".into(), currency_amount_value(fees)?);
        rows.push((transaction.date, transaction.ticker.clone(), row));
    }

    rows.sort_by(|a, b| a.0.cmp(&b.0).then_with(|| a.1.cmp(&b.1)));

    let values: Vec<Value> = rows
        .into_iter()
        .map(|(_, _, row)| row.into_value())
        .collect();

    Ok((!values.is_empty(), values))
}

fn build_asset_event_rows(transactions: &[Transaction]) -> Result<(bool, Vec<Value>), PdfError> {
    let mut rows: Vec<(NaiveDate, String, Dict)> = Vec::new();

    for transaction in transactions {
        let mut row = Dict::new();
        let (op_type, amount, value) = match &transaction.operation {
            Operation::Dividend {
                amount,
                total_value,
                ..
            } => (
                "DIVIDEND",
                decimal_to_value(*amount)?,
                currency_amount_value(total_value)?,
            ),
            Operation::CapReturn {
                amount,
                total_value,
                ..
            } => (
                "CAPRETURN",
                decimal_to_value(*amount)?,
                currency_amount_value(total_value)?,
            ),
            Operation::Split { ratio } => ("SPLIT", decimal_to_value(*ratio)?, Value::None),
            Operation::Unsplit { ratio } => ("UNSPLIT", decimal_to_value(*ratio)?, Value::None),
            _ => continue,
        };

        row.insert("date".into(), date_dict(transaction.date).into_value());
        row.insert("type".into(), op_type.into_value());
        row.insert("ticker".into(), transaction.ticker.clone().into_value());
        row.insert("amount".into(), amount);
        row.insert("value".into(), value);
        rows.push((transaction.date, transaction.ticker.clone(), row));
    }

    rows.sort_by(|a, b| a.0.cmp(&b.0).then_with(|| a.1.cmp(&b.1)));

    let values: Vec<Value> = rows
        .into_iter()
        .map(|(_, _, row)| row.into_value())
        .collect();

    Ok((!values.is_empty(), values))
}

fn build_disposal_dict(disposal: &Disposal) -> Result<Dict, PdfError> {
    let mut dict = Dict::new();

    let total_gain = disposal.net_gain_or_loss();
    let total_cost = disposal.total_allowable_cost();

    dict.insert("ticker".into(), disposal.ticker.clone().into_value());
    dict.insert("date".into(), date_dict(disposal.date).into_value());
    dict.insert("quantity".into(), decimal_to_value(disposal.quantity)?);
    dict.insert(
        "gross_proceeds".into(),
        decimal_to_value(disposal.gross_proceeds)?,
    );
    dict.insert("proceeds".into(), decimal_to_value(disposal.proceeds)?);
    dict.insert("total_gain".into(), decimal_to_value(total_gain)?);
    dict.insert("total_cost".into(), decimal_to_value(total_cost)?);

    let matches: Vec<Value> = disposal
        .matches
        .iter()
        .map(|m| {
            let mut match_dict = Dict::new();
            match_dict.insert("rule".into(), match_rule_label(&m.rule).into_value());
            match_dict.insert("quantity".into(), decimal_to_value(m.quantity)?);
            match_dict.insert("allowable_cost".into(), decimal_to_value(m.allowable_cost)?);
            match_dict.insert(
                "acquisition_date".into(),
                optional_date_value(m.acquisition_date),
            );
            Ok(match_dict.into_value())
        })
        .collect::<Result<Vec<_>, PdfError>>()?;
    dict.insert("matches".into(), matches.into_value());

    Ok(dict)
}

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
/// Returns `PdfError::TypstCompilation` or `PdfError::PdfExport` if generation fails.
/// Returns `CgtError::UnsupportedExemptionYear` if a tax year's exemption is unavailable.
pub fn format(report: &TaxReport, transactions: &[Transaction]) -> Result<Vec<u8>, PdfError> {
    let data = build_template_data(report, transactions)?;

    let engine = TypstEngine::builder()
        .main_file(TEMPLATE)
        .fonts([ROBOTO_REGULAR, ROBOTO_BOLD])
        .build();

    let compiled = engine.compile_with_input(data);
    let doc = compiled
        .output
        .map_err(|e| PdfError::TypstCompilation(e.to_string()))?;

    let pdf = typst_pdf::pdf(&doc, &typst_pdf::PdfOptions::default())
        .map_err(|e| PdfError::PdfExport(format!("{e:?}")))?;

    Ok(pdf)
}

#[cfg(test)]
mod tests {
    use super::*;
    use cgt_core::{Disposal, Match, MatchRule, TaxPeriod, TaxReport, TaxYearSummary};
    use chrono::NaiveDate;
    use rust_decimal::Decimal;
    use typst::foundations::Value;

    #[test]
    fn test_summary_rows_include_counts_and_totals() {
        let date = NaiveDate::from_ymd_opt(2024, 6, 15).expect("valid date");
        let disposal = Disposal {
            date,
            ticker: "ACME".to_string(),
            quantity: Decimal::from(10),
            gross_proceeds: Decimal::from(1000),
            proceeds: Decimal::from(995),
            matches: vec![Match {
                rule: MatchRule::SameDay,
                quantity: Decimal::from(10),
                allowable_cost: Decimal::from(900),
                gain_or_loss: Decimal::from(95),
                acquisition_date: Some(date),
            }],
        };

        let report = TaxReport {
            tax_years: vec![TaxYearSummary {
                period: TaxPeriod::new(2024).expect("valid tax year"),
                disposals: vec![disposal],
                total_gain: Decimal::from(95),
                total_loss: Decimal::ZERO,
                net_gain: Decimal::from(95),
            }],
            holdings: vec![],
        };

        let rows = build_summary_rows(&report).expect("summary rows");
        let first = rows.first().expect("summary row");
        assert!(matches!(first, Value::Dict(_)), "expected dict summary row");
        let Value::Dict(dict) = first else {
            return;
        };

        assert!(dict.get("disposal_count").is_ok());
        assert!(dict.get("net_gain").is_ok());
        assert!(dict.get("total_gain").is_ok());
        assert!(dict.get("total_loss").is_ok());
    }
}
