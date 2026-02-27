//! Tests for cgt-formatter-plain lib.rs (plain text report formatting)

#![allow(clippy::expect_used)]

use cgt_core::{
    Currency, CurrencyAmount, Disposal, Match, MatchRule, Operation, TaxPeriod, TaxReport,
    TaxYearSummary, Transaction,
};
use cgt_format::{format_date, format_gbp};
use cgt_formatter_plain::format;
use chrono::NaiveDate;
use rust_decimal::Decimal;

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
        gross_proceeds: Decimal::new(46702, 3), // 10 × 4.6702
        proceeds: Decimal::new(34202, 3),       // gross - 12.50 fees
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
    assert!(output.contains("Gross Proceeds: 10 × £4.6702 = £46.70"));
    assert!(output.contains("Net Proceeds: £46.70 - £12.50 fees = £34.20"));
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
