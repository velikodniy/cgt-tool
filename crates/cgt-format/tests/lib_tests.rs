//! Tests for cgt-format lib.rs (currency and date formatting)

#![allow(clippy::expect_used)]

use cgt_format::{
    format_currency_amount, format_date, format_decimal_trimmed, format_decimal_with_precision,
    format_gbp, format_price, format_tax_year,
};
use cgt_money::{Currency, CurrencyAmount};
use chrono::NaiveDate;
use rust_decimal::Decimal;

#[test]
fn test_format_gbp_positive() {
    assert_eq!(format_gbp(Decimal::from(100)), "£100.00");
    assert_eq!(format_gbp(Decimal::from(1234)), "£1,234.00");
    assert_eq!(format_gbp(Decimal::from(1000000)), "£1,000,000.00");
}

#[test]
fn test_format_gbp_negative() {
    assert_eq!(format_gbp(Decimal::from(-20)), "-£20.00");
    assert_eq!(format_gbp(Decimal::from(-1234)), "-£1,234.00");
    assert_eq!(format_gbp(Decimal::new(-196, 1)), "-£19.60");
}

#[test]
fn test_format_gbp_zero() {
    assert_eq!(format_gbp(Decimal::ZERO), "£0.00");
}

#[test]
fn test_format_gbp_rounds_decimals() {
    assert_eq!(format_gbp(Decimal::new(10099, 2)), "£100.99");
    assert_eq!(format_gbp(Decimal::new(100999, 3)), "£101.00");
    assert_eq!(format_gbp(Decimal::new(-100999, 3)), "-£101.00");
}

#[test]
fn test_format_decimal_with_precision() {
    assert_eq!(
        format_decimal_with_precision(Decimal::new(1234, 2), 2),
        "12.34"
    );
    assert_eq!(
        format_decimal_with_precision(Decimal::new(1234, 2), 4),
        "12.3400"
    );
    assert_eq!(
        format_decimal_with_precision(Decimal::new(-56789, 3), 2),
        "-56.79"
    );
}

#[test]
fn test_format_decimal_trimmed() {
    assert_eq!(format_decimal_trimmed(Decimal::from(100)), "100");
    assert_eq!(format_decimal_trimmed(Decimal::new(1234, 1)), "123.4");
    assert_eq!(format_decimal_trimmed(Decimal::new(12300, 2)), "123");
    assert_eq!(format_decimal_trimmed(Decimal::new(12340, 2)), "123.4");
}

#[test]
fn test_format_date() {
    let date = NaiveDate::from_ymd_opt(2018, 8, 28).expect("valid date");
    assert_eq!(format_date(date), "28/08/2018");

    let date = NaiveDate::from_ymd_opt(2024, 1, 5).expect("valid date");
    assert_eq!(format_date(date), "05/01/2024");
}

#[test]
fn test_format_tax_year() {
    assert_eq!(format_tax_year(2023), "2023/24");
    assert_eq!(format_tax_year(2014), "2014/15");
    assert_eq!(format_tax_year(2099), "2099/00");
}

#[test]
fn test_format_currency_amount_gbp() {
    let amount = CurrencyAmount::new(Decimal::new(12345, 2), Currency::GBP);
    assert_eq!(format_currency_amount(&amount), "£123.45");
}

#[test]
fn test_format_price_with_precision() {
    let amount = CurrencyAmount::new(Decimal::new(46702, 4), Currency::GBP);
    assert_eq!(format_price(&amount), "£4.6702");
}

#[test]
fn test_format_price_trims_zeros() {
    let amount = CurrencyAmount::new(Decimal::new(12500, 2), Currency::GBP);
    assert_eq!(format_price(&amount), "£125");
}
