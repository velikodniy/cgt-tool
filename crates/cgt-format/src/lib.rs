//! Shared formatting utilities for currency, dates, and numbers.
//!
//! This crate provides consistent formatting across all output formats (plain text, PDF).
//! All formatting uses UK conventions (£ symbol, DD/MM/YYYY dates, thousands separators).

use cgt_money::CurrencyAmount;
use chrono::NaiveDate;
use rust_decimal::{Decimal, RoundingStrategy};

/// Format a `CurrencyAmount` rounded to its minor units (e.g., pence for GBP).
///
/// Used for totals, proceeds, and costs where values should be rounded.
/// GBP amounts display with £ symbol, non-GBP amounts show value with currency code.
///
/// # Examples
/// ```
/// use rust_decimal::Decimal;
/// use cgt_money::{Currency, CurrencyAmount};
/// use cgt_format::format_currency_amount;
///
/// let gbp = CurrencyAmount::new(Decimal::new(12345, 2), Currency::GBP);
/// assert_eq!(format_currency_amount(&gbp), "£123.45");
/// ```
pub fn format_currency_amount(amount: &CurrencyAmount) -> String {
    if amount.is_gbp() {
        format_gbp(amount.amount)
    } else {
        let value = format_decimal_with_precision(amount.amount, amount.minor_units() as u32);
        format!("{} {}", value, amount.code())
    }
}

/// Format a `CurrencyAmount` as a unit price with full precision.
///
/// Used for per-share prices where exact values matter.
/// Displays currency symbol with all significant digits, trimming trailing zeros.
///
/// # Examples
/// ```
/// use rust_decimal::Decimal;
/// use cgt_money::{Currency, CurrencyAmount};
/// use cgt_format::format_price;
///
/// let gbp = CurrencyAmount::new(Decimal::new(46702, 4), Currency::GBP);
/// assert_eq!(format_price(&gbp), "£4.6702");
/// ```
pub fn format_price(amount: &CurrencyAmount) -> String {
    let symbol = amount.symbol();
    let value = format_decimal_trimmed(amount.amount);
    if symbol.is_empty() {
        format!("{}{}", amount.code(), value)
    } else {
        format!("{}{}", symbol, value)
    }
}

/// Format a decimal value as GBP with thousands separators and 2 decimal places.
///
/// Uses UK convention: negative values display as `-£100.00` (sign before symbol).
///
/// # Examples
/// ```
/// use rust_decimal::Decimal;
/// use cgt_format::format_gbp;
///
/// assert_eq!(format_gbp(Decimal::from(1234)), "£1,234.00");
/// assert_eq!(format_gbp(Decimal::from(-100)), "-£100.00");
/// ```
pub fn format_gbp(value: Decimal) -> String {
    format_with_symbol_and_precision(value, '£', 2)
}

/// Format a decimal value with a currency symbol, thousands separators, and specified precision.
fn format_with_symbol_and_precision(value: Decimal, symbol: char, minor_units: u32) -> String {
    let rounded = value.round_dp_with_strategy(minor_units, RoundingStrategy::MidpointAwayFromZero);
    let abs_str = format_decimal_with_precision(rounded.abs(), minor_units);
    let mut parts = abs_str.split('.');
    let integer_part = parts.next().unwrap_or("0");
    let fractional_part = parts.next();
    let formatted_int = add_thousands_separators(integer_part);

    let formatted = if let Some(frac) = fractional_part {
        format!("{formatted_int}.{frac}")
    } else {
        formatted_int
    };

    if rounded.is_sign_negative() {
        format!("-{symbol}{formatted}")
    } else {
        format!("{symbol}{formatted}")
    }
}

/// Add thousands separators (commas) to a decimal integer string.
fn add_thousands_separators(integer: &str) -> String {
    let chars: Vec<char> = integer.chars().collect();
    let mut result = String::with_capacity(chars.len() + chars.len() / 3);
    for (i, c) in chars.iter().enumerate() {
        if i > 0 && (chars.len() - i).is_multiple_of(3) {
            result.push(',');
        }
        result.push(*c);
    }
    result
}

/// Format a decimal value with a fixed number of decimal places.
pub fn format_decimal_with_precision(value: Decimal, precision: u32) -> String {
    let rounded = value.round_dp_with_strategy(precision, RoundingStrategy::MidpointAwayFromZero);
    format!("{rounded:.precision$}", precision = precision as usize)
}

/// Format a decimal value, removing trailing zeros and decimal point if unnecessary.
///
/// # Examples
/// ```
/// use rust_decimal::Decimal;
/// use cgt_format::format_decimal_trimmed;
///
/// assert_eq!(format_decimal_trimmed(Decimal::new(1234, 1)), "123.4");
/// assert_eq!(format_decimal_trimmed(Decimal::new(12300, 2)), "123");
/// ```
pub fn format_decimal_trimmed(value: Decimal) -> String {
    let s = value.to_string();
    if s.contains('.') {
        s.trim_end_matches('0').trim_end_matches('.').to_string()
    } else {
        s
    }
}

/// Format a date as DD/MM/YYYY (UK convention).
///
/// # Examples
/// ```
/// use chrono::NaiveDate;
/// use cgt_format::format_date;
///
/// let date = NaiveDate::from_ymd_opt(2024, 3, 15).unwrap();
/// assert_eq!(format_date(date), "15/03/2024");
/// ```
pub fn format_date(date: NaiveDate) -> String {
    date.format("%d/%m/%Y").to_string()
}

/// Format a UK tax year as "YYYY/YY".
///
/// # Examples
/// ```
/// use cgt_format::format_tax_year;
///
/// assert_eq!(format_tax_year(2023), "2023/24");
/// assert_eq!(format_tax_year(2014), "2014/15");
/// ```
pub fn format_tax_year(start_year: u16) -> String {
    format!("{}/{:02}", start_year, (start_year + 1) % 100)
}
