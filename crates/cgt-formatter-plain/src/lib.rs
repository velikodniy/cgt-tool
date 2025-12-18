//! Plain text formatter for CGT tax reports.

use cgt_core::{CgtError, Disposal, MatchRule, Operation, TaxReport, Transaction, get_exemption};
use cgt_format::{
    format_currency_amount, format_date, format_decimal_trimmed, format_gbp, format_price,
    format_tax_year,
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
    let _ = writeln!(
        out,
        "{:<12}{:<12}{:<12}{:<14}Taxable gain",
        "Tax year", "Gain", "Proceeds", "Exemption"
    );
    let _ = writeln!(
        out,
        "=============================================================="
    );

    for year in &report.tax_years {
        let exemption = get_exemption(year.period.start_year())?;
        let proceeds: Decimal = year.disposals.iter().map(|d| d.proceeds).sum();
        let taxable = (year.net_gain - exemption).max(Decimal::ZERO);

        let _ = writeln!(
            out,
            "{:<12}{:<12}{:<12}{:<14}{}",
            format_tax_year(year.period.start_year()),
            format_gbp(year.net_gain),
            format_gbp(proceeds),
            format_gbp(exemption),
            format_gbp(taxable)
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
                format_decimal_trimmed(h.quantity),
                format_decimal_trimmed(cost_basis.round_dp(2))
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
                    format_decimal_trimmed(cost_per_share.round_dp(2))
                );
            }
        }
    }

    // Calculation
    let (sell_price, sell_fees) = transactions
        .iter()
        .find_map(|t| {
            if t.ticker == disposal.ticker && t.date == disposal.date {
                if let Operation::Sell { price, fees, .. } = &t.operation {
                    Some((price.amount, fees.amount))
                } else {
                    None
                }
            } else {
                None
            }
        })
        .unwrap_or_else(|| {
            if disposal.quantity != Decimal::ZERO {
                (disposal.proceeds / disposal.quantity, Decimal::ZERO)
            } else {
                (Decimal::ZERO, Decimal::ZERO)
            }
        });

    if sell_fees > Decimal::ZERO {
        let _ = writeln!(
            out,
            "   Proceeds: {} × £{} - {} fees = {}",
            format_decimal_trimmed(disposal.quantity),
            format_decimal_trimmed(sell_price),
            format_gbp(sell_fees),
            format_gbp(disposal.proceeds)
        );
    } else {
        let _ = writeln!(
            out,
            "   Proceeds: {} × £{} = {}",
            format_decimal_trimmed(disposal.quantity),
            format_decimal_trimmed(sell_price),
            format_gbp(disposal.proceeds)
        );
    }

    let total_cost: Decimal = disposal.matches.iter().map(|m| m.allowable_cost).sum();
    let _ = writeln!(out, "   Cost: {}", format_gbp(total_cost));
    let _ = writeln!(out, "   Result: {}\n", format_gbp(total_gain));
}

#[cfg(test)]
#[allow(clippy::expect_used)]
mod tests {
    use super::*;
    use cgt_core::{
        Currency, CurrencyAmount, Disposal, Match, MatchRule, Operation, TaxPeriod, TaxReport,
        TaxYearSummary,
    };
    use chrono::NaiveDate;

    #[test]
    fn test_format_gbp() {
        assert_eq!(format_gbp(Decimal::from(100)), "£100.00");
        assert_eq!(format_gbp(Decimal::new(-196, 1)), "-£19.60");
    }

    #[test]
    fn test_format_date() {
        let date = NaiveDate::from_ymd_opt(2018, 8, 28).expect("valid date");
        assert_eq!(format_date(date), "28/08/2018");
    }

    #[test]
    fn test_proceeds_line_with_fees() {
        let date = NaiveDate::from_ymd_opt(2018, 8, 28).expect("valid date");
        let ticker = "GB00B41YBW71".to_string();
        let disposal = Disposal {
            date,
            ticker: ticker.clone(),
            quantity: Decimal::from(10),
            proceeds: Decimal::new(34202, 3),
            matches: vec![Match {
                rule: MatchRule::SameDay,
                quantity: Decimal::from(10),
                allowable_cost: Decimal::new(54065, 3),
                gain_or_loss: Decimal::new(-19863, 3),
                acquisition_date: None,
            }],
        };

        let report = TaxReport {
            tax_years: vec![TaxYearSummary {
                period: TaxPeriod::new(2018).expect("valid tax year"),
                disposals: vec![disposal],
                total_gain: Decimal::ZERO,
                total_loss: Decimal::new(19863, 3),
                net_gain: Decimal::new(-19863, 3),
            }],
            holdings: vec![],
        };

        let transactions = vec![Transaction {
            date,
            ticker,
            operation: Operation::Sell {
                amount: Decimal::from(10),
                price: CurrencyAmount::new(Decimal::new(46702, 4), Currency::GBP),
                fees: CurrencyAmount::new(Decimal::new(125, 1), Currency::GBP),
            },
        }];

        let output = format(&report, &transactions).expect("format should succeed");
        assert!(output.contains("Proceeds: 10 × £4.6702 - £12.50 fees = £34.20"));
    }

    #[test]
    fn test_dividend_single_symbol() {
        let date = NaiveDate::from_ymd_opt(2020, 4, 1).expect("valid date");
        let report = TaxReport {
            tax_years: vec![TaxYearSummary {
                period: TaxPeriod::new(2020).expect("valid tax year"),
                disposals: vec![],
                total_gain: Decimal::ZERO,
                total_loss: Decimal::ZERO,
                net_gain: Decimal::ZERO,
            }],
            holdings: vec![],
        };

        let transactions = vec![Transaction {
            date,
            ticker: "FOOBAR".to_string(),
            operation: Operation::Dividend {
                amount: Decimal::from(15),
                total_value: CurrencyAmount::new(Decimal::new(3000, 2), Currency::GBP),
                tax_paid: CurrencyAmount::new(Decimal::ZERO, Currency::GBP),
            },
        }];

        let output = format(&report, &transactions).expect("format should succeed");
        assert!(output.contains("DIVIDEND FOOBAR 15 £30.00"));
        assert!(!output.contains("££"));
    }
}
