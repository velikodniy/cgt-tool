//! Plain text formatter for CGT tax reports.

use cgt_core::{CgtError, Disposal, MatchRule, Operation, TaxReport, Transaction, get_exemption};
use cgt_format::{
    format_currency_amount, format_date, format_decimal_trimmed, format_gbp, format_price,
    format_tax_year, round_gbp,
};
use rust_decimal::Decimal;
use std::fmt::Write;

/// Format a tax report as plain text.
///
/// # Errors
/// Returns `CgtError::UnsupportedExemptionYear` if the tax year is not supported.
pub fn format(report: &TaxReport, transactions: &[Transaction]) -> Result<String, CgtError> {
    let mut out = String::new();

    // SUMMARY
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
        let exemption = get_exemption(year.period.start_year())?;
        // Use gross proceeds for SA108 Box 21 compatibility
        let gross_proceeds: Decimal = year.disposals.iter().map(|d| d.gross_proceeds).sum();
        let taxable = (year.net_gain - exemption).max(Decimal::ZERO);

        let row_line = format!(
            "{:<12}{:<12}{:<22}{:<22}{:<12}{:<12}{:<12}{:<14}",
            format_tax_year(year.period.start_year()),
            year.disposal_count,
            format_gbp(year.net_gain),
            format_gbp(year.total_gain),
            format_gbp(year.total_loss),
            format_gbp(gross_proceeds),
            format_gbp(exemption),
            format_gbp(taxable)
        );
        let _ = writeln!(out, "{}", row_line.trim_end());
    }

    // SA108 note
    if !report.tax_years.is_empty() && report.tax_years.iter().any(|y| !y.disposals.is_empty()) {
        let _ = writeln!(
            out,
            "\nNotes:\n- Disposal count groups same-day disposals into a single transaction (CG51560) and may differ from raw SELL transactions\n- Gains/Losses are net per disposal after matching rules (CG51560)\n- Proceeds = SA108 Box 21 (gross, before sale fees)"
        );
    }

    // TAX YEAR DETAILS
    let _ = writeln!(out, "\n# TAX YEAR DETAILS");

    for year in &report.tax_years {
        let _ = writeln!(out, "\n## {}\n", format_tax_year(year.period.start_year()));

        // Sort disposals by date, then by ticker for deterministic output
        let mut disposals: Vec<_> = year.disposals.iter().collect();
        disposals.sort_by(|a, b| a.date.cmp(&b.date).then_with(|| a.ticker.cmp(&b.ticker)));

        for (i, disposal) in disposals.iter().enumerate() {
            format_disposal(&mut out, i + 1, disposal);
        }
    }

    // HOLDINGS
    let _ = writeln!(out, "\n# HOLDINGS\n");
    let mut active: Vec<_> = report
        .holdings
        .iter()
        .filter(|h| h.quantity > Decimal::ZERO)
        .collect();
    active.sort_by(|a, b| a.ticker.cmp(&b.ticker));
    if active.is_empty() {
        let _ = writeln!(out, "NONE");
    } else {
        for h in active {
            let cost_basis = h.total_cost / h.quantity;
            let _ = writeln!(
                out,
                "{}: {} units at £{} avg cost",
                h.ticker,
                format_decimal_trimmed(h.quantity),
                format_decimal_trimmed(round_gbp(cost_basis))
            );
        }
    }

    // TRANSACTIONS
    let _ = writeln!(out, "\n# TRANSACTIONS\n");
    let mut txns: Vec<_> = transactions
        .iter()
        .filter(|t| matches!(t.operation, Operation::Buy { .. } | Operation::Sell { .. }))
        .collect();
    txns.sort_by(|a, b| a.date.cmp(&b.date).then_with(|| a.ticker.cmp(&b.ticker)));

    for t in txns {
        match &t.operation {
            Operation::Buy {
                amount,
                price,
                fees,
            } => {
                let _ = writeln!(
                    out,
                    "{} BUY {} {} @ {} ({} fees)",
                    format_date(t.date),
                    format_decimal_trimmed(*amount),
                    t.ticker,
                    format_price(price),
                    format_price(fees)
                );
            }
            Operation::Sell {
                amount,
                price,
                fees,
            } => {
                let _ = writeln!(
                    out,
                    "{} SELL {} {} @ {} ({} fees)",
                    format_date(t.date),
                    format_decimal_trimmed(*amount),
                    t.ticker,
                    format_price(price),
                    format_price(fees)
                );
            }
            _ => {}
        }
    }

    // ASSET EVENTS
    let mut events: Vec<_> = transactions
        .iter()
        .filter(|t| {
            matches!(
                t.operation,
                Operation::Dividend { .. }
                    | Operation::CapReturn { .. }
                    | Operation::Split { .. }
                    | Operation::Unsplit { .. }
            )
        })
        .collect();
    events.sort_by(|a, b| a.date.cmp(&b.date).then_with(|| a.ticker.cmp(&b.ticker)));

    if !events.is_empty() {
        let _ = writeln!(out, "\n# ASSET EVENTS\n");
        for t in events {
            match &t.operation {
                Operation::Dividend {
                    amount,
                    total_value,
                    ..
                } => {
                    let _ = writeln!(
                        out,
                        "{} DIVIDEND {} {} {}",
                        format_date(t.date),
                        t.ticker,
                        format_decimal_trimmed(*amount),
                        format_currency_amount(total_value)
                    );
                }
                Operation::CapReturn {
                    amount,
                    total_value,
                    ..
                } => {
                    let _ = writeln!(
                        out,
                        "{} CAPRETURN {} {} {}",
                        format_date(t.date),
                        t.ticker,
                        format_decimal_trimmed(*amount),
                        format_currency_amount(total_value)
                    );
                }
                Operation::Split { ratio } => {
                    let _ = writeln!(
                        out,
                        "{} SPLIT {} {}",
                        format_date(t.date),
                        t.ticker,
                        format_decimal_trimmed(*ratio)
                    );
                }
                Operation::Unsplit { ratio } => {
                    let _ = writeln!(
                        out,
                        "{} UNSPLIT {} {}",
                        format_date(t.date),
                        t.ticker,
                        format_decimal_trimmed(*ratio)
                    );
                }
                _ => {}
            }
        }
    }

    Ok(out.trim_end().to_string() + "\n")
}

fn format_disposal(out: &mut String, index: usize, disposal: &Disposal) {
    let total_gain: Decimal = disposal.matches.iter().map(|m| m.gain_or_loss).sum();
    let gain_type = if total_gain >= Decimal::ZERO {
        "GAIN"
    } else {
        "LOSS"
    };

    let _ = writeln!(
        out,
        "{}) SELL {} {} on {} - {} {}",
        index,
        format_decimal_trimmed(disposal.quantity),
        disposal.ticker,
        format_date(disposal.date),
        gain_type,
        format_gbp(total_gain.abs())
    );

    for m in &disposal.matches {
        match m.rule {
            MatchRule::SameDay => {
                let _ = writeln!(
                    out,
                    "   Same Day: {} shares",
                    format_decimal_trimmed(m.quantity)
                );
            }
            MatchRule::BedAndBreakfast => {
                if let Some(date) = m.acquisition_date {
                    let _ = writeln!(
                        out,
                        "   B&B: {} shares from {}",
                        format_decimal_trimmed(m.quantity),
                        format_date(date)
                    );
                }
            }
            MatchRule::Section104 => {
                let cost_per_share = if m.quantity != Decimal::ZERO {
                    m.allowable_cost / m.quantity
                } else {
                    Decimal::ZERO
                };
                let _ = writeln!(
                    out,
                    "   Section 104: {} shares @ £{}",
                    format_decimal_trimmed(m.quantity),
                    format_decimal_trimmed(round_gbp(cost_per_share))
                );
            }
        }
    }

    // Calculate unit price from gross proceeds (handles same-day merges correctly)
    let unit_price = if disposal.quantity != Decimal::ZERO {
        disposal.gross_proceeds / disposal.quantity
    } else {
        Decimal::ZERO
    };

    // Calculate fees from difference between gross and net
    let sell_fees = disposal.gross_proceeds - disposal.proceeds;

    // Show gross proceeds calculation
    let _ = writeln!(
        out,
        "   Gross Proceeds: {} × £{} = {}",
        format_decimal_trimmed(disposal.quantity),
        format_decimal_trimmed(unit_price),
        format_gbp(disposal.gross_proceeds)
    );

    // Show net proceeds if fees exist
    if sell_fees > Decimal::ZERO {
        let _ = writeln!(
            out,
            "   Net Proceeds: {} - {} fees = {}",
            format_gbp(disposal.gross_proceeds),
            format_gbp(sell_fees),
            format_gbp(disposal.proceeds)
        );
    }

    let total_cost: Decimal = disposal.matches.iter().map(|m| m.allowable_cost).sum();
    let _ = writeln!(out, "   Cost: {}", format_gbp(total_cost));
    let _ = writeln!(out, "   Result: {}\n", format_gbp(total_gain));
}
