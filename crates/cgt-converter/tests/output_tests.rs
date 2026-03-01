//! Tests for cgt-converter output.rs (DSL line formatting)

#![allow(clippy::expect_used)]

use cgt_converter::output::{format_comment, format_dividend, format_trade};
use chrono::NaiveDate;
use rust_decimal_macros::dec;

#[test]
fn test_format_trade_buy_without_expenses() {
    let date = NaiveDate::from_ymd_opt(2023, 4, 25).unwrap();
    let result = format_trade(
        "BUY",
        &date,
        "XYZZ",
        dec!(67.2),
        dec!(125.6445),
        "USD",
        None,
    );
    assert_eq!(result, "2023-04-25 BUY XYZZ 67.2 @ 125.6445 USD");
}

#[test]
fn test_format_trade_buy_with_expenses() {
    let date = NaiveDate::from_ymd_opt(2023, 5, 10).unwrap();
    let result = format_trade(
        "BUY",
        &date,
        "XYZZ",
        dec!(10),
        dec!(130.00),
        "USD",
        Some(dec!(4.95)),
    );
    assert_eq!(result, "2023-05-10 BUY XYZZ 10 @ 130.00 USD FEES 4.95 USD");
}

#[test]
fn test_format_trade_sell() {
    let date = NaiveDate::from_ymd_opt(2023, 6, 14).unwrap();
    let result = format_trade(
        "SELL",
        &date,
        "XYZZ",
        dec!(62.601495),
        dec!(113.75),
        "USD",
        Some(dec!(0.17)),
    );
    assert_eq!(
        result,
        "2023-06-14 SELL XYZZ 62.601495 @ 113.75 USD FEES 0.17 USD"
    );
}

#[test]
fn test_format_dividend_with_tax() {
    let date = NaiveDate::from_ymd_opt(2023, 7, 15).unwrap();
    let result = format_dividend(&date, "FOO", dec!(50.00), "USD", Some(dec!(7.50)));
    assert_eq!(
        result,
        "2023-07-15 DIVIDEND FOO TOTAL 50.00 USD TAX 7.50 USD"
    );
}

#[test]
fn test_format_comment() {
    assert_eq!(format_comment("Test comment"), "# Test comment");
}
