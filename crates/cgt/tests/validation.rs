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
        line: None,
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
        line: None,
    }
}

fn make_split(date: &str, ticker: &str, ratio: i64) -> Transaction {
    Transaction {
        date: date.parse().expect("valid date"),
        ticker: ticker.to_string(),
        operation: Operation::Split {
            ratio: Decimal::from(ratio),
        },
        line: None,
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
        line: None,
    }
}

fn make_capreturn(date: &str, ticker: &str, amount: i64, total: i64) -> Transaction {
    Transaction {
        date: date.parse().expect("valid date"),
        ticker: ticker.to_string(),
        operation: Operation::CapReturn {
            amount: Decimal::from(amount),
            total_value: CurrencyAmount::new(Decimal::from(total), Currency::GBP),
            fees: CurrencyAmount::new(Decimal::ZERO, Currency::GBP),
        },
        line: None,
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
fn test_same_date_split_and_trade_same_ticker_is_rejected() {
    let txns = vec![
        make_buy("2024-01-01", "ABC", 100, 10, 0),
        make_split("2024-06-01", "ABC", 2),
        make_sell("2024-06-01", "ABC", 50, 6, 0),
    ];
    let result = validate(&txns);
    assert!(!result.is_valid());
    assert!(
        result.errors[0].message.contains("ambiguous"),
        "got: {}",
        result.errors[0].message
    );
}

#[test]
fn test_same_date_split_and_accumulation_same_ticker_is_allowed() {
    // ACCUMULATION adjusts only cost (the engine discards its quantity), so it
    // commutes with a same-date split and is not ambiguous. Restore the guard
    // if ACCUMULATION ever becomes unit-issuing.
    let txns = vec![
        make_buy("2024-01-01", "ABC", 100, 10, 0),
        make_split("2024-06-01", "ABC", 2),
        make_accumulation("2024-06-01", "ABC", 40, 30),
    ];
    assert!(
        validate(&txns).is_valid(),
        "accumulation carries no split-ordering ambiguity"
    );
}

#[test]
fn test_same_date_split_and_capreturn_same_ticker_is_allowed() {
    // A CAPRETURN changes no share count, so its order relative to a split is
    // immaterial: it must not be rejected.
    let txns = vec![
        make_buy("2024-01-01", "ABC", 100, 10, 0),
        make_split("2024-06-01", "ABC", 2),
        make_capreturn("2024-06-01", "ABC", 200, 50),
    ];
    let result = validate(&txns);
    assert!(
        result.is_valid(),
        "a capital return carries no split-ordering ambiguity"
    );
}

#[test]
fn test_unsplit_non_integer_ratio_is_rejected() {
    let txns = vec![Transaction {
        date: "2024-06-01".parse().expect("valid date"),
        ticker: "ABC".to_string(),
        operation: Operation::Unsplit {
            ratio: Decimal::new(15, 1), // 1.5
        },
        line: None,
    }];
    let result = validate(&txns);
    assert!(!result.is_valid());
    assert!(
        result
            .errors
            .iter()
            .any(|e| e.message.contains("non-integer ratio"))
    );
}

#[test]
fn test_split_and_trade_different_ticker_same_date_is_allowed() {
    let txns = vec![
        make_buy("2024-01-01", "ABC", 100, 10, 0),
        make_split("2024-06-01", "ABC", 2),
        make_sell("2024-06-01", "XYZ", 50, 6, 0),
    ];
    let result = validate(&txns);
    assert!(result.is_valid(), "different tickers must not collide");
}

#[test]
fn test_split_and_trade_different_date_same_ticker_is_allowed() {
    let txns = vec![
        make_buy("2024-01-01", "ABC", 100, 10, 0),
        make_split("2024-06-01", "ABC", 2),
        make_sell("2024-06-02", "ABC", 50, 6, 0),
    ];
    let result = validate(&txns);
    assert!(result.is_valid(), "different dates must not collide");
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
    // The BUY exists (2020-06-01), just later, so the accurate message is
    // "SELL before first BUY" — the old "no prior BUY" pinned the bug.
    assert!(result.warnings[0].message.contains("SELL before first BUY"));
}

#[test]
fn test_sell_listed_before_chronologically_earlier_buy_is_not_warned() {
    // SELL appears first in input but its BUY is chronologically earlier: no
    // spurious warning, because first-buy is computed over all transactions.
    let txns = vec![
        make_sell("2020-06-20", "AAPL", 50, 180, 10),
        make_buy("2020-01-15", "AAPL", 100, 150, 10),
    ];
    assert!(validate(&txns).is_clean());
}

#[test]
fn test_sell_with_no_buy() {
    let txns = vec![make_sell("2020-01-01", "AAPL", 50, 180, 10)];
    let result = validate(&txns);
    assert!(result.is_valid());
    assert_eq!(result.warnings.len(), 1);
    assert!(result.warnings[0].message.contains("no prior BUY"));
}
