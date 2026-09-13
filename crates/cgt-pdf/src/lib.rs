//! PDF renderer for CGT tax reports using embedded Typst.
//!
//! Generates a professional PDF from a tax report without any external tool
//! installation. Depends only on the engine crate so non-PDF builds never pull
//! in Typst.

use cgt::format::{format_currency_amount, format_decimal_trimmed, format_gbp, format_price};
use cgt::model::{CurrencyAmount, Operation, Transaction};
use cgt::report::{Disposal, MatchRule, TaxReport};
use chrono::{Datelike, Local, NaiveDate};
use rust_decimal::Decimal;
use thiserror::Error;
use typst::foundations::{Dict, IntoValue, Value};
use typst::layout::PagedDocument;
use typst_as_lib::TypstEngine;

static TEMPLATE: &str = include_str!("templates/report.typ");

// Roboto font (Apache 2.0 license; see fonts/LICENSE.txt).
static ROBOTO_REGULAR: &[u8] = include_bytes!("../fonts/Roboto-Regular.ttf");
static ROBOTO_BOLD: &[u8] = include_bytes!("../fonts/Roboto-Bold.ttf");

#[derive(Debug, Error)]
pub enum PdfError {
    #[error("PDF generation failed: unit value exceeds the supported range")]
    UnitValueOverflow,

    #[error("PDF generation failed: Typst compilation failed: {0}")]
    TypstCompilation(String),

    #[error("PDF generation failed: PDF export failed: {0}")]
    PdfExport(String),
}

fn decimal_to_value(value: Decimal) -> Result<Value, PdfError> {
    Ok(format_decimal_trimmed(value).into_value())
}

/// Format money before passing it to Typst so no value passes through f64.
fn money_to_value(value: Decimal) -> Result<Value, PdfError> {
    Ok(format_gbp(value).into_value())
}

fn unit_amount(total: Decimal, quantity: Decimal) -> Result<Decimal, PdfError> {
    if quantity.is_zero() {
        Ok(Decimal::ZERO)
    } else {
        total
            .checked_div(quantity)
            .ok_or(PdfError::UnitValueOverflow)
    }
}

/// Per-share value `total / quantity`, rounded as money (0 when quantity is 0).
fn unit_value(total: Decimal, quantity: Decimal) -> Result<Value, PdfError> {
    money_to_value(unit_amount(total, quantity)?)
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
    Ok(format_currency_amount(amount).into_value())
}

fn match_rule_label(rule: &MatchRule) -> &'static str {
    match rule {
        MatchRule::SameDay => "SAME_DAY",
        MatchRule::BedAndBreakfast => "BED_AND_BREAKFAST",
        MatchRule::Section104 => "SECTION_104",
    }
}

fn build_template_data(report: &TaxReport) -> Result<Dict, PdfError> {
    let mut data = Dict::new();

    let today = Local::now().date_naive();
    data.insert("generation_date".into(), date_dict(today).into_value());

    let summary_rows = build_summary_rows(report)?;
    data.insert("summary_rows".into(), summary_rows.into_value());

    let tax_years = build_tax_years(report)?;
    data.insert("tax_years".into(), tax_years.into_value());

    let (has_holdings, holdings_rows) = build_holdings_rows(report)?;
    data.insert("has_holdings".into(), has_holdings.into_value());
    data.insert("holdings_rows".into(), holdings_rows.into_value());

    let echo = report.transactions.as_deref().unwrap_or(&[]);

    let (has_transactions, transaction_rows) = build_transaction_rows(echo)?;
    data.insert("has_transactions".into(), has_transactions.into_value());
    data.insert("transaction_rows".into(), transaction_rows.into_value());

    let (has_asset_events, asset_event_rows) = build_asset_event_rows(echo)?;
    data.insert("has_asset_events".into(), has_asset_events.into_value());
    data.insert("asset_event_rows".into(), asset_event_rows.into_value());

    Ok(data)
}

fn build_summary_rows(report: &TaxReport) -> Result<Vec<Value>, PdfError> {
    let mut rows = Vec::new();
    for year in &report.tax_years {
        let mut row = Dict::new();
        row.insert(
            "start_year".into(),
            (year.period.start_year() as i64).into_value(),
        );
        row.insert(
            "disposal_count".into(),
            (year.disposal_count as i64).into_value(),
        );
        row.insert("net_gain".into(), money_to_value(year.net_gain)?);
        row.insert("total_gain".into(), money_to_value(year.total_gain)?);
        row.insert("total_loss".into(), money_to_value(year.total_loss)?);
        row.insert(
            "gross_proceeds".into(),
            money_to_value(year.gross_proceeds)?,
        );
        row.insert("exemption".into(), money_to_value(year.exempt_amount)?);
        row.insert("taxable".into(), money_to_value(year.taxable_gain)?);
        row.insert(
            "dividend_income".into(),
            money_to_value(year.dividend_income)?,
        );
        row.insert(
            "dividend_tax_paid".into(),
            money_to_value(year.dividend_tax_paid)?,
        );
        row.insert(
            "has_dividend_totals".into(),
            (!year.dividend_income.is_zero() || !year.dividend_tax_paid.is_zero()).into_value(),
        );
        row.insert(
            "has_dividend_tax".into(),
            (!year.dividend_tax_paid.is_zero()).into_value(),
        );
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

            let disposals: Vec<Value> = year
                .disposals
                .iter()
                .map(build_disposal_dict)
                .map(|result| result.map(IntoValue::into_value))
                .collect::<Result<Vec<_>, PdfError>>()?;

            year_dict.insert("disposals".into(), disposals.into_value());
            Ok(year_dict.into_value())
        })
        .collect()
}

fn build_holdings_rows(report: &TaxReport) -> Result<(bool, Vec<Value>), PdfError> {
    let active = report.active_holdings();

    let rows: Vec<Value> = active
        .iter()
        .map(|h| {
            let mut row = Dict::new();
            row.insert("ticker".into(), h.ticker.clone().into_value());
            row.insert("quantity".into(), decimal_to_value(h.quantity)?);
            row.insert("total_cost".into(), money_to_value(h.total_cost)?);
            row.insert("avg_cost".into(), unit_value(h.total_cost, h.quantity)?);
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
        row.insert("price".into(), format_price(price).into_value());
        row.insert("fees".into(), format_price(fees).into_value());
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
            Operation::Dividend { total_value, .. } => {
                ("DIVIDEND", Value::None, currency_amount_value(total_value)?)
            }
            Operation::Accumulation {
                amount,
                total_value,
                ..
            } => (
                "ACCUMULATION",
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

    let total_gain = disposal.total_gain();
    let total_cost = disposal.total_allowable_cost();

    dict.insert("ticker".into(), disposal.ticker.clone().into_value());
    dict.insert("date".into(), date_dict(disposal.date).into_value());
    dict.insert("quantity".into(), decimal_to_value(disposal.quantity)?);
    dict.insert(
        "gross_proceeds".into(),
        money_to_value(disposal.gross_proceeds)?,
    );
    dict.insert("proceeds".into(), money_to_value(disposal.proceeds)?);
    dict.insert("total_gain".into(), money_to_value(total_gain)?);
    dict.insert("gain_magnitude".into(), money_to_value(total_gain.abs())?);
    dict.insert("is_gain".into(), (total_gain >= Decimal::ZERO).into_value());
    dict.insert("total_cost".into(), money_to_value(total_cost)?);
    let adjustment = disposal.rounding_adjustment();
    dict.insert(
        "has_rounding_adjustment".into(),
        (!adjustment.is_zero()).into_value(),
    );
    dict.insert("rounding_adjustment".into(), money_to_value(adjustment)?);
    dict.insert(
        "unit_price".into(),
        format!(
            "£{}",
            format_decimal_trimmed(unit_amount(disposal.gross_proceeds, disposal.quantity)?)
        )
        .into_value(),
    );
    dict.insert(
        "has_fees".into(),
        (disposal.gross_proceeds > disposal.proceeds).into_value(),
    );
    dict.insert(
        "fees".into(),
        money_to_value(
            cgt::format::round_money(disposal.gross_proceeds)
                - cgt::format::round_money(disposal.proceeds),
        )?,
    );

    let matches: Vec<Value> = disposal
        .display_legs()
        .iter()
        .map(|m| {
            let mut match_dict = Dict::new();
            match_dict.insert("rule".into(), match_rule_label(&m.rule).into_value());
            match_dict.insert("quantity".into(), decimal_to_value(m.quantity)?);
            match_dict.insert("allowable_cost".into(), money_to_value(m.allowable_cost)?);
            match_dict.insert(
                "unit_cost".into(),
                unit_value(m.allowable_cost, m.quantity)?,
            );
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

/// Render a PDF report from calculated tax data.
///
/// # Errors
/// Returns [`PdfError`] if a unit value exceeds the supported range, or if Typst
/// compilation or PDF export fails.
pub fn render(report: &TaxReport) -> Result<Vec<u8>, PdfError> {
    let doc = compile(report)?;
    let pdf = typst_pdf::pdf(&doc, &typst_pdf::PdfOptions::default())
        .map_err(|e| PdfError::PdfExport(format!("{e:?}")))?;

    Ok(pdf)
}

fn compile(report: &TaxReport) -> Result<PagedDocument, PdfError> {
    let data = build_template_data(report)?;

    let engine = TypstEngine::builder()
        .main_file(TEMPLATE)
        .fonts([ROBOTO_REGULAR, ROBOTO_BOLD])
        .build();

    let compiled = engine.compile_with_input(data);
    compiled
        .output
        .map_err(|e| PdfError::TypstCompilation(e.to_string()))
}

#[cfg(test)]
mod tests {
    use super::{compile, render};
    use cgt::{Config, calculate};
    use std::path::{Path, PathBuf};
    use typst::layout::{Frame, FrameItem};

    fn append_frame_text(frame: &Frame, text: &mut String) {
        for (_, item) in frame.items() {
            match item {
                FrameItem::Text(item) => {
                    text.push_str(&item.text);
                    text.push(' ');
                }
                FrameItem::Group(item) => append_frame_text(&item.frame, text),
                _ => {}
            }
        }
    }

    fn rendered_text(dsl: &str) -> Result<String, Box<dyn std::error::Error>> {
        let transactions = cgt::dsl::parse(dsl)?;
        let config = Config::embedded()?;
        let report = calculate(&transactions, None, None, &config)?;
        let document = compile(&report)?;
        let mut text = String::new();
        for page in &document.pages {
            append_frame_text(&page.frame, &mut text);
        }
        Ok(text.split_whitespace().collect::<Vec<_>>().join(" "))
    }

    #[test]
    fn rendered_quantities_and_unit_prices_keep_their_precision()
    -> Result<(), Box<dyn std::error::Error>> {
        let text = rendered_text(
            "2024-01-01 BUY TINY 0.0000001 @ 10000000\n\
             2024-06-01 SELL TINY 0.0000001 @ 20000000\n\
             2024-01-01 BUY UNIT 100 @ 1.001\n\
             2024-06-01 SELL UNIT 100 @ 1.005\n",
        )?;
        assert!(text.contains("0.0000001 shares"), "{text}");
        assert!(text.contains("100 × £1.005 = £100.50"), "{text}");
        assert!(text.contains("£1.001"), "{text}");
        Ok(())
    }

    #[test]
    fn rendered_disposal_explains_independent_rounding() -> Result<(), Box<dyn std::error::Error>> {
        let text = rendered_text(include_str!("../../../tests/inputs/DisposalRounding.cgt"))?;
        assert!(text.contains("Rounding adjustment"), "{text}");
        assert!(text.contains("£0.01"), "{text}");
        assert!(text.contains("£0.02"), "{text}");
        Ok(())
    }

    #[test]
    fn rendered_money_keeps_pennies_above_float_precision() -> Result<(), Box<dyn std::error::Error>>
    {
        let text = rendered_text(
            "2024-01-01 BUY LARGE 1 @ 9007199254740992.01\n\
             2024-06-01 SELL LARGE 1 @ 9007199254740992.99\n\
             2024-06-01 DIVIDEND LARGE TOTAL 1.005\n",
        )?;
        assert!(text.contains("£9,007,199,254,740,992.99"), "{text}");
        assert!(text.contains("£0.98"), "{text}");
        assert!(text.contains("£1.01"), "{text}");
        Ok(())
    }

    #[test]
    fn rendered_summary_includes_dividend_income_and_tax() -> Result<(), Box<dyn std::error::Error>>
    {
        let text = rendered_text(
            "2024-01-01 BUY FUND 1 @ 1\n\
             2024-06-01 SELL FUND 1 @ 2\n\
             2024-06-01 DIVIDEND FUND TOTAL 100 TAX 15\n",
        )?;
        assert!(text.contains("Dividend income 2024/25: £100.00"), "{text}");
        assert!(text.contains("tax paid: £15.00"), "{text}");
        Ok(())
    }

    #[test]
    fn rendered_summary_includes_tax_without_dividend_income()
    -> Result<(), Box<dyn std::error::Error>> {
        let text = rendered_text("2024-06-01 DIVIDEND A TOTAL 0 TAX 1\n")?;
        assert!(text.contains("Dividend income 2024/25: £0.00"), "{text}");
        assert!(text.contains("tax paid: £1.00"), "{text}");
        Ok(())
    }

    #[test]
    fn rendered_sale_fees_reconcile_displayed_proceeds() -> Result<(), Box<dyn std::error::Error>> {
        let text = rendered_text(
            "2024-01-01 BUY FEE 1 @ 0.5\n\
             2024-06-01 SELL FEE 1 @ 1.004 FEES 0.005\n",
        )?;
        assert!(
            text.contains("Net Proceeds: £1.00 − £0.00 = £1.00"),
            "{text}"
        );
        Ok(())
    }

    fn fixture(name: &str) -> PathBuf {
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("..")
            .join("..")
            .join("tests")
            .join("inputs")
            .join(format!("{name}.cgt"))
    }

    fn report_for(name: &str) -> cgt::TaxReport {
        let content = std::fs::read_to_string(fixture(name)).expect("fixture readable");
        let transactions = cgt::dsl::parse(&content).expect("fixture parses");
        let fx = cgt::money::FxRates::bundled();
        let config = Config::embedded().expect("embedded config loads");
        calculate(&transactions, None, Some(&fx), &config).expect("report builds")
    }

    fn fixture_names() -> Vec<String> {
        let dir = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../tests/inputs");
        let mut names: Vec<String> = std::fs::read_dir(&dir)
            .expect("read tests/inputs")
            .filter_map(Result::ok)
            .filter_map(|e| {
                let p = e.path();
                (p.extension().is_some_and(|x| x == "cgt")).then(|| {
                    p.file_stem()
                        .unwrap_or_default()
                        .to_string_lossy()
                        .into_owned()
                })
            })
            .collect();
        names.sort();
        names
    }

    #[test]
    fn every_fixture_renders_a_valid_pdf() {
        let names = fixture_names();
        assert!(names.len() >= 40, "fixture discovery found too few inputs");
        for name in &names {
            let pdf = render(&report_for(name))
                .unwrap_or_else(|e| panic!("{name}: PDF render failed: {e:?}"));
            assert!(pdf.starts_with(b"%PDF-"), "{name}: output is not a PDF");
        }
    }

    #[test]
    fn renders_a_substantive_pdf_for_a_complex_fixture() {
        let pdf = render(&report_for("SyntheticComplex")).expect("PDF renders");
        assert!(pdf.len() > 1000, "PDF has substantive content");
    }
}
