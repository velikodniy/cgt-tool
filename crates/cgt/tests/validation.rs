//! Tests for cgt validate.rs (transaction validation)

#![allow(clippy::expect_used)]

use cgt::model::{Currency, CurrencyAmount, Operation, Transaction};
use cgt::validate;
use rust_decimal::Decimal;

fn make_buy(date: &str, ticker: &str, amount: i64, price: i64, fees: i64) -> Transaction {
    Transaction {
        date: date.parse().expect("valid date"),
        ticker: ticker.to_string(),
        operation: Operation::Buy {
            amount: Decimal::from(amount),
            price: CurrencyAmount::new(Decimal::from(price), Currency::GBP),
            fees: CurrencyAmount::new(Decimal::from(fees), Currency::GBP),
        },
    }
}

fn make_sell(date: &str, ticker: &str, amount: i64, price: i64, fees: i64) -> Transaction {
    Transaction {
        date: date.parse().expect("valid date"),
        ticker: ticker.to_string(),
        operation: Operation::Sell {
            amount: Decimal::from(amount),
            price: CurrencyAmount::new(Decimal::from(price), Currency::GBP),
            fees: CurrencyAmount::new(Decimal::from(fees), Currency::GBP),
        },
    }
}

fn make_split(date: &str, ticker: &str, ratio: i64) -> Transaction {
    Transaction {
        date: date.parse().expect("valid date"),
        ticker: ticker.to_string(),
        operation: Operation::Split {
            ratio: Decimal::from(ratio),
        },
    }
}

fn make_accumulation(date: &str, ticker: &str, amount: i64, total: i64) -> Transaction {
    Transaction {
        date: date.parse().expect("valid date"),
        ticker: ticker.to_string(),
        operation: Operation::Accumulation {
            amount: Decimal::from(amount),
            total_value: CurrencyAmount::new(Decimal::from(total), Currency::GBP),
            tax_paid: CurrencyAmount::new(Decimal::ZERO, Currency::GBP),
        },
    }
}

#[test]
fn test_zero_quantity_accumulation_is_accepted() {
    // A zero-quantity accumulation is a no-op (no units accrue), not an error.
    let txns = vec![
        make_buy("2020-01-01", "FUND", 100, 10, 0),
        make_accumulation("2020-06-01", "FUND", 0, 0),
    ];
    let result = validate::validate(&txns);
    assert!(
        result.is_valid(),
        "zero-quantity accumulation must be accepted"
    );
}

#[test]
fn test_negative_quantity_accumulation_is_rejected() {
    let txns = vec![make_accumulation("2020-06-01", "FUND", -5, 10)];
    let result = validate::validate(&txns);
    assert!(!result.is_valid());
    assert!(
        result.errors[0].message.contains("negative quantity"),
        "got: {}",
        result.errors[0].message
    );
}

#[test]
fn test_valid_transactions() {
    let txns = vec![
        make_buy("2020-01-01", "AAPL", 100, 150, 10),
        make_sell("2020-06-01", "AAPL", 50, 180, 10),
    ];
    let result = validate(&txns);
    assert!(result.is_valid());
    assert!(result.is_clean());
}

#[test]
fn test_zero_quantity_buy() {
    let txns = vec![make_buy("2020-01-01", "AAPL", 0, 150, 10)];
    let result = validate(&txns);
    assert!(!result.is_valid());
    assert_eq!(result.errors.len(), 1);
    assert!(result.errors[0].message.contains("zero quantity"));
}

#[test]
fn test_zero_quantity_sell() {
    let txns = vec![
        make_buy("2020-01-01", "AAPL", 100, 150, 10),
        make_sell("2020-06-01", "AAPL", 0, 180, 10),
    ];
    let result = validate(&txns);
    assert!(!result.is_valid());
    assert_eq!(result.errors.len(), 1);
    assert!(result.errors[0].message.contains("zero quantity"));
}

#[test]
fn test_negative_price() {
    let txns = vec![make_buy("2020-01-01", "AAPL", 100, -150, 10)];
    let result = validate(&txns);
    assert!(!result.is_valid());
    assert!(result.errors[0].message.contains("negative price"));
}

#[test]
fn test_negative_fees() {
    let txns = vec![make_buy("2020-01-01", "AAPL", 100, 150, -10)];
    let result = validate(&txns);
    assert!(!result.is_valid());
    assert!(result.errors[0].message.contains("negative fees"));
}

#[test]
fn test_zero_split_ratio() {
    let txns = vec![make_split("2020-01-01", "AAPL", 0)];
    let result = validate(&txns);
    assert!(!result.is_valid());
    assert!(result.errors[0].message.contains("zero ratio"));
}

#[test]
fn test_negative_split_ratio() {
    let txns = vec![make_split("2020-01-01", "AAPL", -2)];
    let result = validate(&txns);
    assert!(!result.is_valid());
    assert!(result.errors[0].message.contains("negative ratio"));
}

#[test]
fn test_sell_before_buy_warning() {
    let txns = vec![
        make_sell("2020-01-01", "AAPL", 50, 180, 10),
        make_buy("2020-06-01", "AAPL", 100, 150, 10),
    ];
    let result = validate(&txns);
    // Valid (just a warning)
    assert!(result.is_valid());
    // But not clean
    assert!(!result.is_clean());
    assert_eq!(result.warnings.len(), 1);
    assert!(result.warnings[0].message.contains("no prior BUY"));
}

#[test]
fn test_sell_with_no_buy() {
    let txns = vec![make_sell("2020-01-01", "AAPL", 50, 180, 10)];
    let result = validate(&txns);
    assert!(result.is_valid());
    assert_eq!(result.warnings.len(), 1);
    assert!(result.warnings[0].message.contains("no prior BUY"));
}
