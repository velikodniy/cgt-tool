//! Fixed-width plain-text layout of a prepared report.

use std::fmt::Write;

use crate::report::MatchRule;
use crate::report_display::{DisplayedDisposal, ReportDisplay, ResultKind};

/// Render a prepared tax report as plain text.
pub fn render(report: &ReportDisplay) -> String {
    let report = report.data();
    let mut out = String::new();
    let _ = writeln!(out, "# SUMMARY\n");
    let header_line = format!(
        "{:<12}{:<12}{:<22}{:<22}{:<12}{:<12}{:<12}{:<14}",
        "Tax year",
        "Disposals",
        "Gains (after losses)",
        "Gains (before losses)",
        "Losses",
        "Proceeds",
        "Exemption",
        "Taxable gain"
    );
    let _ = writeln!(out, "{}", header_line.trim_end());
    let _ = writeln!(
        out,
        "========================================================================================================================="
    );
    for year in &report.tax_years {
        let row_line = format!(
            "{:<12}{:<12}{:<22}{:<22}{:<12}{:<12}{:<12}{:<14}",
            year.period,
            year.disposal_count,
            year.net_gain,
            year.total_gain,
            year.total_loss,
            year.gross_proceeds,
            year.exempt_amount,
            year.taxable_gain
        );
        let _ = writeln!(out, "{}", row_line.trim_end());
        if let Some(dividend) = &year.dividend {
            if let Some(tax) = &dividend.tax_paid {
                let _ = writeln!(
                    out,
                    "Dividend income: {} (tax paid: {})",
                    dividend.income, tax
                );
            } else {
                let _ = writeln!(out, "Dividend income: {}", dividend.income);
            }
        }
    }
    if !report.notes.is_empty() {
        let _ = writeln!(out, "\nNotes:");
        for note in &report.notes {
            let _ = writeln!(out, "- {}", note.text);
        }
    }
    let _ = writeln!(out, "\n# TAX YEAR DETAILS");
    for year in &report.tax_years {
        let _ = writeln!(out, "\n## {}\n", year.period);
        for (i, disposal) in year.disposals.iter().enumerate() {
            format_disposal(&mut out, i + 1, disposal);
        }
    }
    let _ = writeln!(out, "\n# HOLDINGS\n");
    if report.holdings.is_empty() {
        let _ = writeln!(out, "NONE");
    } else {
        for holding in &report.holdings {
            let _ = writeln!(
                out,
                "{}: {} units at {} avg cost",
                holding.ticker, holding.quantity, holding.unit_cost.compact
            );
        }
    }
    let _ = writeln!(out, "\n# TRANSACTIONS\n");
    for trade in &report.trades {
        let _ = writeln!(
            out,
            "{} {} {} {} @ {} ({} fees)",
            trade.date.uk, trade.action, trade.quantity, trade.ticker, trade.price, trade.fees
        );
    }
    if !report.asset_events.is_empty() {
        let _ = writeln!(out, "\n# ASSET EVENTS\n");
        for event in &report.asset_events {
            let _ = write!(out, "{} {} {}", event.date.uk, event.action, event.ticker);
            if let Some(amount) = &event.amount {
                let _ = write!(out, " {amount}");
            }
            if let Some(value) = &event.value {
                let _ = write!(out, " {value}");
            }
            let _ = writeln!(out);
        }
    }
    out.trim_end().to_string() + "\n"
}

fn format_disposal(out: &mut String, index: usize, disposal: &DisplayedDisposal) {
    let gain_type = match disposal.result.kind {
        ResultKind::Gain => "GAIN",
        ResultKind::Loss => "LOSS",
    };
    let _ = writeln!(
        out,
        "{}) SELL {} {} on {} - {} {}",
        index,
        disposal.quantity,
        disposal.ticker,
        disposal.date.uk,
        gain_type,
        disposal.result.magnitude
    );
    for leg in &disposal.matches {
        match leg.rule {
            MatchRule::SameDay => {
                let _ = writeln!(out, "   Same Day: {} shares", leg.quantity);
            }
            MatchRule::BedAndBreakfast => {
                if let Some(date) = &leg.acquisition_date {
                    let _ = writeln!(out, "   B&B: {} shares from {}", leg.quantity, date.uk);
                }
            }
            MatchRule::Section104 => {
                let _ = writeln!(
                    out,
                    "   Section 104: {} shares @ {}",
                    leg.quantity, leg.unit_cost.compact
                );
            }
        }
    }
    let _ = writeln!(
        out,
        "   Gross Proceeds: {} × {} = {}",
        disposal.quantity, disposal.sale_unit_price, disposal.gross_proceeds
    );
    if let Some(fees) = &disposal.fees {
        let _ = writeln!(
            out,
            "   Net Proceeds: {} - {} fees = {}",
            disposal.gross_proceeds, fees, disposal.proceeds
        );
    }
    let _ = writeln!(out, "   Cost: {}", disposal.allowable_cost);
    if let Some(adjustment) = &disposal.rounding_adjustment {
        let _ = writeln!(out, "   Rounding adjustment: {adjustment}");
    }
    let _ = writeln!(out, "   Result: {}\n", disposal.result.signed);
}

#[cfg(test)]
mod tests {
    use super::render;
    use crate::{Config, calculate, dsl};

    #[test]
    fn unit_costs_round_once_for_holdings_and_section_104() -> Result<(), Box<dyn std::error::Error>>
    {
        let transactions = dsl::parse(
            "2024-01-01 BUY HOLD 200.00000000000000000000000001 @ 0.005\n\
             2024-01-01 BUY SOLD 200.00000000000000000000000001 @ 0.005\n\
             2024-06-01 SELL SOLD 200.00000000000000000000000001 @ 0.01\n\
             2024-01-01 BUY FRACTION 0.1 @ 100.005\n",
        )?;
        let report = calculate(&transactions, None, None, &Config::embedded()?)?;
        let text = render(&crate::report_display::prepare(&report)?);
        assert!(
            text.contains("HOLD: 200.00000000000000000000000001 units at £0 avg cost"),
            "{text}"
        );
        assert!(
            text.contains("Section 104: 200.00000000000000000000000001 shares @ £0\n"),
            "{text}"
        );
        assert!(
            text.contains("FRACTION: 0.1 units at £100.01 avg cost"),
            "{text}"
        );
        Ok(())
    }
}
