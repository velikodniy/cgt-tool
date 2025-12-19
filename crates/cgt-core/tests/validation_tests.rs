//! Tests for cgt-core validation.rs (transaction validation)

#![allow(clippy::expect_used)]

use cgt_core::models::{CurrencyAmount, Operation, Transaction};
use cgt_core::validate;
use cgt_money::Currency;
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
