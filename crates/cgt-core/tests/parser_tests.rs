use cgt_core::parser::parse_file;
#[allow(unused_imports)] // Clippy false positive
use cgt_core::models::{Transaction, Operation};
use rust_decimal::Decimal;
use chrono::NaiveDate;
use std::str::FromStr; // Needed for Decimal::from_str

#[test]
fn test_parse_valid_buy() {
    let input = "2023-01-01 BUY AAPL 10 150.00 5.00";
    let transactions = parse_file(input).expect("Failed to parse valid BUY transaction");
    assert_eq!(transactions.len(), 1);
    let tx = &transactions[0];
    assert_eq!(tx.date, NaiveDate::from_ymd_opt(2023, 1, 1).unwrap());
    assert_eq!(tx.ticker, "AAPL");
    if let Operation::Buy { amount, price, expenses } = &tx.operation {
        assert_eq!(*amount, Decimal::from(10));
        assert_eq!(*price, Decimal::from_str("150.00").unwrap());
        assert_eq!(*expenses, Decimal::from_str("5.00").unwrap());
    } else {
        panic!("Expected Buy operation");
    }
}
