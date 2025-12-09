//! Shared formatting utilities for currency, dates, and numbers.
//!
//! This module provides consistent formatting across all output formats (plain text, PDF).

use chrono::NaiveDate;
use rust_decimal::Decimal;

/// Policy for formatting values in reports.
///
/// Currently uses UK conventions. Future versions may support different locales.
#[derive(Debug, Clone, Default)]
pub struct FormattingPolicy {
    /// Currency symbol (default: £)
    pub currency_symbol: char,
    /// Date format string (default: %d/%m/%Y)
    pub date_format: String,
    /// Use thousands separators in currency (default: true)
    pub use_thousands_separator: bool,
}

impl FormattingPolicy {
    /// Create a new formatting policy with UK defaults.
    pub fn uk() -> Self {
        Self {
            currency_symbol: '£',
            date_format: "%d/%m/%Y".to_string(),
            use_thousands_separator: true,
        }
    }
}

/// Format a decimal value as currency with thousands separators.
///
/// Uses UK convention: negative values display as `-£100` (sign before symbol).
///
/// # Examples
/// ```
/// use rust_decimal::Decimal;
/// use cgt_core::formatting::format_currency;
///
/// assert_eq!(format_currency(Decimal::from(1234)), "£1,234");
/// assert_eq!(format_currency(Decimal::from(-100)), "-£100");
/// ```
pub fn format_currency(value: Decimal) -> String {
    let floored = value.floor();
    let abs_value = floored.abs();
    let formatted = format_with_commas(abs_value);
    if floored < Decimal::ZERO {
        format!("-£{formatted}")
    } else {
        format!("£{formatted}")
    }
}

/// Add thousands separators to a decimal value.
fn format_with_commas(value: Decimal) -> String {
    let s = value.to_string();
    let integer_part = s.split('.').next().unwrap_or("0");

    let chars: Vec<char> = integer_part.chars().collect();
    let mut result = String::with_capacity(chars.len() + chars.len() / 3);
    for (i, c) in chars.iter().enumerate() {
        if i > 0 && (chars.len() - i) % 3 == 0 {
            result.push(',');
        }
        result.push(*c);
    }
    result
}

/// Format a decimal value, removing trailing zeros after the decimal point.
///
/// # Examples
/// ```
/// use rust_decimal::Decimal;
/// use cgt_core::formatting::format_decimal;
///
/// assert_eq!(format_decimal(Decimal::new(1234, 1)), "123.4");
/// assert_eq!(format_decimal(Decimal::new(12300, 2)), "123");
/// ```
pub fn format_decimal(value: Decimal) -> String {
    let s = value.to_string();
    if s.contains('.') {
        s.trim_end_matches('0').trim_end_matches('.').to_string()
    } else {
        s
    }
}

/// Format a date as DD/MM/YYYY.
///
/// # Examples
/// ```
/// use chrono::NaiveDate;
/// use cgt_core::formatting::format_date;
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
/// use cgt_core::formatting::format_tax_year;
///
/// assert_eq!(format_tax_year(2023), "2023/24");
/// assert_eq!(format_tax_year(2014), "2014/15");
/// ```
pub fn format_tax_year(start_year: u16) -> String {
    format!("{}/{:02}", start_year, (start_year + 1) % 100)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_currency_positive() {
        assert_eq!(format_currency(Decimal::from(100)), "£100");
        assert_eq!(format_currency(Decimal::from(1234)), "£1,234");
        assert_eq!(format_currency(Decimal::from(1000000)), "£1,000,000");
    }

    #[test]
    fn test_format_currency_negative() {
        assert_eq!(format_currency(Decimal::from(-20)), "-£20");
        assert_eq!(format_currency(Decimal::from(-1234)), "-£1,234");
        assert_eq!(format_currency(Decimal::new(-196, 1)), "-£20");
    }

    #[test]
    fn test_format_currency_zero() {
        assert_eq!(format_currency(Decimal::ZERO), "£0");
    }

    #[test]
    fn test_format_currency_floors_decimals() {
        assert_eq!(format_currency(Decimal::new(10099, 2)), "£100");
        assert_eq!(format_currency(Decimal::new(-10099, 2)), "-£101");
    }

    #[test]
    fn test_format_decimal() {
        assert_eq!(format_decimal(Decimal::from(100)), "100");
        assert_eq!(format_decimal(Decimal::new(1234, 1)), "123.4");
        assert_eq!(format_decimal(Decimal::new(12300, 2)), "123");
        assert_eq!(format_decimal(Decimal::new(12340, 2)), "123.4");
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
    fn test_formatting_policy_uk_defaults() {
        let policy = FormattingPolicy::uk();
        assert_eq!(policy.currency_symbol, '£');
        assert_eq!(policy.date_format, "%d/%m/%Y");
        assert!(policy.use_thousands_separator);
    }
}
