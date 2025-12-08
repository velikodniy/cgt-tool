#![allow(clippy::expect_used, clippy::panic)]

use cgt_core::models::Operation;
use cgt_core::parser::parse_file;
use chrono::NaiveDate;
use rust_decimal::Decimal;
use std::str::FromStr;

#[test]
fn test_parse_valid_buy() {
    let input = "2023-01-01 BUY AAPL 10 @ 150.00 EXPENSES 5.00";
    let transactions = parse_file(input).expect("Failed to parse valid BUY transaction");
    assert_eq!(transactions.len(), 1);
    let tx = &transactions[0];
    assert_eq!(
        tx.date,
        NaiveDate::from_ymd_opt(2023, 1, 1).expect("valid test date")
    );
    assert_eq!(tx.ticker, "AAPL");
    if let Operation::Buy {
        amount,
        price,
        expenses,
    } = &tx.operation
    {
        assert_eq!(*amount, Decimal::from(10));
        assert_eq!(*price, Decimal::from_str("150.00").expect("valid decimal"));
        assert_eq!(*expenses, Decimal::from_str("5.00").expect("valid decimal"));
    } else {
        panic!("Expected Buy operation");
    }
}

#[test]
fn test_parse_dividend_with_tax_keyword() {
    let input = "2019-11-30 DIVIDEND GB00B3TYHH97 10 TOTAL 110.93 TAX 0";
    let transactions = parse_file(input).expect("Failed to parse DIVIDEND with TAX keyword");
    assert_eq!(transactions.len(), 1);
    let tx = &transactions[0];
    assert_eq!(
        tx.date,
        NaiveDate::from_ymd_opt(2019, 11, 30).expect("valid test date")
    );
    assert_eq!(tx.ticker, "GB00B3TYHH97");
    if let Operation::Dividend {
        amount,
        total_value,
        tax_paid,
    } = &tx.operation
    {
        assert_eq!(*amount, Decimal::from(10));
        assert_eq!(
            *total_value,
            Decimal::from_str("110.93").expect("valid decimal")
        );
        assert_eq!(*tax_paid, Decimal::from(0));
    } else {
        panic!("Expected Dividend operation");
    }
}

#[test]
fn test_parse_capreturn_with_expenses_keyword() {
    let input = "2019-05-31 CAPRETURN GB00B3TYHH97 10 TOTAL 149.75 EXPENSES 0";
    let transactions = parse_file(input).expect("Failed to parse CAPRETURN with EXPENSES keyword");
    assert_eq!(transactions.len(), 1);
    let tx = &transactions[0];
    assert_eq!(
        tx.date,
        NaiveDate::from_ymd_opt(2019, 5, 31).expect("valid test date")
    );
    assert_eq!(tx.ticker, "GB00B3TYHH97");
    if let Operation::CapReturn {
        amount,
        total_value,
        expenses,
    } = &tx.operation
    {
        assert_eq!(*amount, Decimal::from(10));
        assert_eq!(
            *total_value,
            Decimal::from_str("149.75").expect("valid decimal")
        );
        assert_eq!(*expenses, Decimal::from(0));
    } else {
        panic!("Expected CapReturn operation");
    }
}

#[test]
fn test_parse_split_with_ratio_keyword() {
    let input = "2019-02-15 SPLIT FOO RATIO 2";
    let transactions = parse_file(input).expect("Failed to parse SPLIT with RATIO keyword");
    assert_eq!(transactions.len(), 1);
    let tx = &transactions[0];
    assert_eq!(
        tx.date,
        NaiveDate::from_ymd_opt(2019, 2, 15).expect("valid test date")
    );
    assert_eq!(tx.ticker, "FOO");
    if let Operation::Split { ratio } = &tx.operation {
        assert_eq!(*ratio, Decimal::from(2));
    } else {
        panic!("Expected Split operation");
    }
}

#[test]
fn test_parse_unsplit_with_ratio_keyword() {
    let input = "2019-02-15 UNSPLIT FOO RATIO 2";
    let transactions = parse_file(input).expect("Failed to parse UNSPLIT with RATIO keyword");
    assert_eq!(transactions.len(), 1);
    let tx = &transactions[0];
    assert_eq!(
        tx.date,
        NaiveDate::from_ymd_opt(2019, 2, 15).expect("valid test date")
    );
    assert_eq!(tx.ticker, "FOO");
    if let Operation::Unsplit { ratio } = &tx.operation {
        assert_eq!(*ratio, Decimal::from(2));
    } else {
        panic!("Expected Unsplit operation");
    }
}
