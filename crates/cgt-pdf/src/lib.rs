//! PDF layout and export of prepared reports using embedded Typst.

use cgt::report::MatchRule;
use cgt::report_display::{DisplayedDisposal, ReportDisplay, ResultKind};
use chrono::Local;
use thiserror::Error;
use typst::foundations::{Dict, IntoValue, Value};
use typst_as_lib::TypstEngine;
use typst_layout::PagedDocument;

static TEMPLATE: &str = include_str!("templates/report.typ");

// Roboto font (Apache 2.0 license; see fonts/LICENSE.txt).
static ROBOTO_REGULAR: &[u8] = include_bytes!("../fonts/Roboto-Regular.ttf");
static ROBOTO_BOLD: &[u8] = include_bytes!("../fonts/Roboto-Bold.ttf");

#[derive(Debug, Error)]
pub enum PdfError {
    #[error("PDF generation failed: Typst compilation failed: {0}")]
    TypstCompilation(String),

    #[error("PDF generation failed: PDF export failed: {0}")]
    PdfExport(String),
}

fn text(value: &str) -> Value {
    value.to_owned().into_value()
}

fn optional_text(value: Option<&str>) -> Value {
    value.map(text).unwrap_or(Value::None)
}

fn build_template_data(report: &ReportDisplay) -> Dict {
    let report = report.data();
    let mut data = Dict::new();
    data.insert(
        "generation_date".into(),
        Local::now().format("%d/%m/%Y").to_string().into_value(),
    );
    let mut summary_rows = Vec::new();
    let mut tax_years = Vec::new();
    for year in &report.tax_years {
        let mut row = Dict::new();
        row.insert("period".into(), text(&year.period));
        row.insert("disposal_count".into(), text(&year.disposal_count));
        row.insert("net_gain".into(), text(&year.net_gain));
        row.insert("total_gain".into(), text(&year.total_gain));
        row.insert("total_loss".into(), text(&year.total_loss));
        row.insert("gross_proceeds".into(), text(&year.gross_proceeds));
        row.insert("exemption".into(), text(&year.exempt_amount));
        row.insert("taxable".into(), text(&year.taxable_gain));
        row.insert(
            "has_dividend_totals".into(),
            year.dividend.is_some().into_value(),
        );
        row.insert(
            "has_dividend_tax".into(),
            year.dividend
                .as_ref()
                .is_some_and(|dividend| dividend.tax_paid.is_some())
                .into_value(),
        );
        row.insert(
            "dividend_income".into(),
            optional_text(
                year.dividend
                    .as_ref()
                    .map(|dividend| dividend.income.as_str()),
            ),
        );
        row.insert(
            "dividend_tax_paid".into(),
            optional_text(
                year.dividend
                    .as_ref()
                    .and_then(|dividend| dividend.tax_paid.as_deref()),
            ),
        );
        summary_rows.push(row.into_value());

        let mut details = Dict::new();
        details.insert("period".into(), text(&year.period));
        details.insert(
            "disposals".into(),
            year.disposals
                .iter()
                .map(build_disposal_dict)
                .map(IntoValue::into_value)
                .collect::<Vec<_>>()
                .into_value(),
        );
        tax_years.push(details.into_value());
    }
    data.insert("summary_rows".into(), summary_rows.into_value());
    data.insert("tax_years".into(), tax_years.into_value());

    let holdings: Vec<_> = report
        .holdings
        .iter()
        .map(|holding| {
            let mut row = Dict::new();
            row.insert("ticker".into(), text(&holding.ticker));
            row.insert("quantity".into(), text(&holding.quantity));
            row.insert("avg_cost".into(), text(&holding.unit_cost.money));
            row.into_value()
        })
        .collect();
    data.insert("has_holdings".into(), (!holdings.is_empty()).into_value());
    data.insert("holdings_rows".into(), holdings.into_value());

    let trades: Vec<_> = report
        .trades
        .iter()
        .map(|trade| {
            let mut row = Dict::new();
            row.insert("date".into(), text(&trade.date.uk));
            row.insert("type".into(), text(trade.action));
            row.insert("ticker".into(), text(&trade.ticker));
            row.insert("quantity".into(), text(&trade.quantity));
            row.insert("price".into(), text(&trade.price));
            row.insert("fees".into(), text(&trade.fees));
            row.into_value()
        })
        .collect();
    data.insert("has_transactions".into(), (!trades.is_empty()).into_value());
    data.insert("transaction_rows".into(), trades.into_value());

    let events: Vec<_> = report
        .asset_events
        .iter()
        .map(|event| {
            let mut row = Dict::new();
            row.insert("date".into(), text(&event.date.uk));
            row.insert("type".into(), text(event.action));
            row.insert("ticker".into(), text(&event.ticker));
            row.insert("amount".into(), optional_text(event.amount.as_deref()));
            row.insert("value".into(), optional_text(event.value.as_deref()));
            row.into_value()
        })
        .collect();
    data.insert("has_asset_events".into(), (!events.is_empty()).into_value());
    data.insert("asset_event_rows".into(), events.into_value());

    let mut note_numbers = Dict::new();
    for (index, note) in report.notes.iter().enumerate() {
        note_numbers.insert(note.kind.key().into(), (index + 1).to_string().into_value());
    }
    data.insert("note_numbers".into(), note_numbers.into_value());
    data.insert(
        "notes".into(),
        report
            .notes
            .iter()
            .map(|note| text(note.text))
            .collect::<Vec<_>>()
            .into_value(),
    );
    data
}

fn build_disposal_dict(disposal: &DisplayedDisposal) -> Dict {
    let mut dict = Dict::new();
    dict.insert("ticker".into(), text(&disposal.ticker));
    dict.insert("date".into(), text(&disposal.date.uk));
    dict.insert("quantity".into(), text(&disposal.quantity));
    dict.insert("gross_proceeds".into(), text(&disposal.gross_proceeds));
    dict.insert("proceeds".into(), text(&disposal.proceeds));
    dict.insert("total_gain".into(), text(&disposal.result.signed));
    dict.insert("gain_magnitude".into(), text(&disposal.result.magnitude));
    dict.insert(
        "is_gain".into(),
        (disposal.result.kind == ResultKind::Gain).into_value(),
    );
    dict.insert("total_cost".into(), text(&disposal.allowable_cost));
    dict.insert(
        "has_rounding_adjustment".into(),
        disposal.rounding_adjustment.is_some().into_value(),
    );
    dict.insert(
        "rounding_adjustment".into(),
        optional_text(disposal.rounding_adjustment.as_deref()),
    );
    dict.insert("unit_price".into(), text(&disposal.sale_unit_price));
    dict.insert("has_fees".into(), disposal.fees.is_some().into_value());
    dict.insert("fees".into(), optional_text(disposal.fees.as_deref()));
    let matches: Vec<_> = disposal
        .matches
        .iter()
        .map(|leg| {
            let mut row = Dict::new();
            let rule = match leg.rule {
                MatchRule::SameDay => "SAME_DAY",
                MatchRule::BedAndBreakfast => "BED_AND_BREAKFAST",
                MatchRule::Section104 => "SECTION_104",
            };
            row.insert("rule".into(), text(rule));
            row.insert("quantity".into(), text(&leg.quantity));
            row.insert("unit_cost".into(), text(&leg.unit_cost.money));
            row.insert(
                "acquisition_date".into(),
                optional_text(leg.acquisition_date.as_ref().map(|date| date.uk.as_str())),
            );
            row.into_value()
        })
        .collect();
    dict.insert("matches".into(), matches.into_value());
    dict
}

/// Render a PDF from prepared display data.
///
/// # Errors
/// Returns [`PdfError`] if Typst compilation or PDF export fails.
pub fn render(report: &ReportDisplay) -> Result<Vec<u8>, PdfError> {
    let doc = compile(report)?;
    typst_pdf::pdf(&doc, &typst_pdf::PdfOptions::default())
        .map_err(|error| PdfError::PdfExport(format!("{error:?}")))
}

fn compile(report: &ReportDisplay) -> Result<PagedDocument, PdfError> {
    let engine = TypstEngine::builder()
        .main_file(TEMPLATE)
        .fonts([ROBOTO_REGULAR, ROBOTO_BOLD])
        .build();
    engine
        .compile_with_input(build_template_data(report))
        .output
        .map_err(|error| PdfError::TypstCompilation(error.to_string()))
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
        let document = compile(&cgt::report_display::prepare(&report)?)?;
        let mut text = String::new();
        for page in document.pages() {
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
    fn rendered_unit_costs_round_once_for_holdings_and_section_104()
    -> Result<(), Box<dyn std::error::Error>> {
        let text = rendered_text(
            "2024-01-01 BUY HOLD 200.00000000000000000000000001 @ 0.005\n\
             2024-01-01 BUY SOLD 200.00000000000000000000000001 @ 0.005\n\
             2024-06-01 SELL SOLD 200.00000000000000000000000001 @ 0.01\n\
             2024-01-01 BUY FRACTION 0.1 @ 100.005\n",
        )?;
        assert!(
            text.contains("Section 104: 200.00000000000000000000000001 shares @ £0.00"),
            "{text}"
        );
        assert!(
            text.contains("HOLD 200.00000000000000000000000001 £0.00"),
            "{text}"
        );
        assert!(text.contains("FRACTION 0.1 £100.01"), "{text}");
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

    #[test]
    fn rendered_notes_follow_shared_selection() -> Result<(), Box<dyn std::error::Error>> {
        for input in [
            "2024-01-01 BUY FUND 1 @ 1\n",
            "2024-06-01 DIVIDEND FUND TOTAL 42.5\n",
            "2024-01-01 BUY FUND 1 @ 1\n2024-06-01 SELL FUND 1 @ 2\n",
        ] {
            let report = calculate(&cgt::dsl::parse(input)?, None, None, &Config::embedded()?)?;
            let display = cgt::report_display::prepare(&report)?;
            let text = rendered_text(input)?;
            for note in &display.data().notes {
                assert!(text.contains(note.text), "{text}");
            }
            assert_eq!(text.contains("Notes:"), !display.data().notes.is_empty());
            assert_eq!(
                text.contains("Disposal count groups"),
                display
                    .data()
                    .notes
                    .iter()
                    .any(|note| note.kind == cgt::report_display::NoteKind::DisposalCount)
            );
            assert!(!text.contains("SA108 Box"), "{text}");
        }
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

    fn report_for(
        name: &str,
    ) -> Result<cgt::report_display::ReportDisplay, Box<dyn std::error::Error>> {
        let content = std::fs::read_to_string(fixture(name))?;
        let transactions = cgt::dsl::parse(&content)?;
        let fx = cgt::money::FxRates::bundled();
        let config = Config::embedded()?;
        let report = calculate(&transactions, None, Some(&fx), &config)?;
        Ok(cgt::report_display::prepare(&report)?)
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
    fn every_fixture_renders_a_valid_pdf() -> Result<(), Box<dyn std::error::Error>> {
        let names = fixture_names();
        assert!(names.len() >= 40, "fixture discovery found too few inputs");
        for name in &names {
            let pdf = render(&report_for(name)?)?;
            assert!(pdf.starts_with(b"%PDF-"), "{name}: output is not a PDF");
        }
        Ok(())
    }

    #[test]
    fn renders_a_substantive_pdf_for_a_complex_fixture() -> Result<(), Box<dyn std::error::Error>> {
        let pdf = render(&report_for("SyntheticComplex")?)?;
        assert!(pdf.len() > 1000, "PDF has substantive content");
        Ok(())
    }
}
