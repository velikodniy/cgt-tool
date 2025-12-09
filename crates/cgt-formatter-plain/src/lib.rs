//! Plain text formatter for CGT tax reports.

use cgt_core::{CgtError, Disposal, MatchRule, Operation, TaxReport, Transaction, get_exemption};
use chrono::NaiveDate;
use rust_decimal::Decimal;
use std::fmt::Write;

// =============================================================================
// Helper Functions
// =============================================================================

fn format_currency(value: Decimal) -> String {
    format!("£{}", value.floor())
}

fn format_decimal(value: Decimal) -> String {
    let s = value.to_string();
    if s.contains('.') {
        s.trim_end_matches('0').trim_end_matches('.').to_string()
    } else {
        s
    }
}

fn format_date(date: NaiveDate) -> String {
    date.format("%d/%m/%Y").to_string()
}

fn format_tax_year(start_year: u16) -> String {
    format!("{}/{}", start_year, start_year + 1)
}

// =============================================================================
// Main Format Function
// =============================================================================

/// Format a tax report as plain text.
///
/// # Errors
/// Returns `CgtError::UnsupportedExemptionYear` if the tax year is not supported.
pub fn format(report: &TaxReport, transactions: &[Transaction]) -> Result<String, CgtError> {
    let mut out = String::new();

    // SUMMARY
    let _ = writeln!(out, "# SUMMARY\n");
    let _ = writeln!(
        out,
        "Tax year    Gain   Proceeds   Exemption   Taxable gain"
    );
    let _ = writeln!(
        out,
        "=========================================================="
    );

    for year in &report.tax_years {
        let exemption = get_exemption(year.period.start_year())?;
        let proceeds: Decimal = year.disposals.iter().map(|d| d.proceeds).sum();
        let taxable = (year.net_gain - exemption).max(Decimal::ZERO);

        let _ = writeln!(
            out,
            "{:<12}{:<7}{:<11}{:<12}{}",
            format_tax_year(year.period.start_year()),
            format_currency(year.net_gain),
            format_currency(proceeds),
            format_currency(exemption),
            format_currency(taxable)
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
            format_disposal(&mut out, i + 1, disposal, transactions);
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
                format_decimal(h.quantity),
                format_decimal(cost_basis.round_dp(2))
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
                expenses,
            } => {
                let _ = writeln!(
                    out,
                    "{} BUY {} {} @ £{} (£{} fees)",
                    format_date(t.date),
                    format_decimal(*amount),
                    t.ticker,
                    format_decimal(*price),
                    format_decimal(*expenses)
                );
            }
            Operation::Sell {
                amount,
                price,
                expenses,
            } => {
                let _ = writeln!(
                    out,
                    "{} SELL {} {} @ £{} (£{} fees)",
                    format_date(t.date),
                    format_decimal(*amount),
                    t.ticker,
                    format_decimal(*price),
                    format_decimal(*expenses)
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
                        "{} DIVIDEND {} {} £{}",
                        format_date(t.date),
                        t.ticker,
                        format_decimal(*amount),
                        format_decimal(*total_value)
                    );
                }
                Operation::CapReturn {
                    amount,
                    total_value,
                    ..
                } => {
                    let _ = writeln!(
                        out,
                        "{} CAPRETURN {} {} £{}",
                        format_date(t.date),
                        t.ticker,
                        format_decimal(*amount),
                        format_decimal(*total_value)
                    );
                }
                Operation::Split { ratio } => {
                    let _ = writeln!(
                        out,
                        "{} SPLIT {} {}",
                        format_date(t.date),
                        t.ticker,
                        format_decimal(*ratio)
                    );
                }
                Operation::Unsplit { ratio } => {
                    let _ = writeln!(
                        out,
                        "{} UNSPLIT {} {}",
                        format_date(t.date),
                        t.ticker,
                        format_decimal(*ratio)
                    );
                }
                _ => {}
            }
        }
    }

    Ok(out.trim_end().to_string() + "\n")
}

fn format_disposal(
    out: &mut String,
    index: usize,
    disposal: &Disposal,
    transactions: &[Transaction],
) {
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
        format_decimal(disposal.quantity),
        disposal.ticker,
        format_date(disposal.date),
        gain_type,
        format_currency(total_gain.abs())
    );

    for m in &disposal.matches {
        match m.rule {
            MatchRule::SameDay => {
                let _ = writeln!(out, "   Same Day: {} shares", format_decimal(m.quantity));
            }
            MatchRule::BedAndBreakfast => {
                if let Some(date) = m.acquisition_date {
                    let _ = writeln!(
                        out,
                        "   B&B: {} shares from {}",
                        format_decimal(m.quantity),
                        format_date(date)
                    );
                }
            }
            MatchRule::Section104 => {
                let cost_per_share = m.allowable_cost / m.quantity;
                let _ = writeln!(
                    out,
                    "   Section 104: {} shares @ £{}",
                    format_decimal(m.quantity),
                    format_decimal(cost_per_share.round_dp(2))
                );
            }
        }
    }

    // Calculation
    let sell_price = transactions
        .iter()
        .find_map(|t| {
            if t.ticker == disposal.ticker && t.date == disposal.date {
                if let Operation::Sell { price, .. } = &t.operation {
                    Some(*price)
                } else {
                    None
                }
            } else {
                None
            }
        })
        .unwrap_or(disposal.proceeds / disposal.quantity);

    let _ = writeln!(
        out,
        "   Proceeds: {} × £{} = {}",
        format_decimal(disposal.quantity),
        format_decimal(sell_price),
        format_currency(disposal.proceeds)
    );

    let total_cost: Decimal = disposal.matches.iter().map(|m| m.allowable_cost).sum();
    let _ = writeln!(out, "   Cost: {}", format_currency(total_cost));
    let _ = writeln!(out, "   Result: {}\n", format_currency(total_gain));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_currency() {
        assert_eq!(format_currency(Decimal::from(100)), "£100");
        assert_eq!(format_currency(Decimal::new(-196, 1)), "£-20");
    }

    #[test]
    fn test_format_date() {
        let date = NaiveDate::from_ymd_opt(2018, 8, 28).expect("valid date");
        assert_eq!(format_date(date), "28/08/2018");
    }
}
