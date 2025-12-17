//! Shared formatting utilities for currency, dates, and numbers.
//!
//! This crate provides consistent formatting across all output formats (plain text, PDF).

use cgt_money::CurrencyAmount;
use chrono::NaiveDate;
use rust_decimal::{Decimal, RoundingStrategy};

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

/// Currency formatter with configurable rounding.
///
/// Provides methods for formatting `CurrencyAmount` values with proper
/// symbol handling, thousands separators, and rounding behavior.
#[derive(Debug, Clone)]
pub struct CurrencyFormatter {
    #[allow(dead_code)] // Reserved for future locale-specific formatting
    policy: FormattingPolicy,
}

impl CurrencyFormatter {
    /// Create a new currency formatter with UK defaults.
    pub fn uk() -> Self {
        Self {
            policy: FormattingPolicy::uk(),
        }
    }

    /// Format a `CurrencyAmount` as GBP, rounded to currency minor units.
    ///
    /// For totals, proceeds, costs - values that should be shown rounded.
    /// Shows original currency in parentheses only if it's not GBP.
    pub fn format_amount(&self, amount: &CurrencyAmount) -> String {
        if amount.is_gbp() {
            format_currency(amount.amount)
        } else {
            let orig = format_decimal_fixed(amount.amount, amount.minor_units() as u32);
            format!("{} {}", orig, amount.code())
        }
    }

    /// Format a `CurrencyAmount` preserving full precision.
    ///
    /// For unit prices where precision matters in transaction breakdowns.
    /// Uses currency symbol (or ISO code as fallback) with trimmed decimals.
    pub fn format_unit(&self, amount: &CurrencyAmount) -> String {
        let symbol = amount.symbol();
        let value = format_decimal(amount.amount);
        if symbol.is_empty() {
            format!("{}{}", amount.code(), value)
        } else {
            format!("{}{}", symbol, value)
        }
    }

    /// Format a raw decimal as GBP currency.
    pub fn format_decimal(&self, value: Decimal) -> String {
        format_currency(value)
    }
}

impl Default for CurrencyFormatter {
    fn default() -> Self {
        Self::uk()
    }
}

/// Format a decimal value as currency with thousands separators and minor units.
///
/// Uses UK convention: negative values display as `-£100.00` (sign before symbol)
/// and values are rounded to two decimal places (GBP minor units).
///
/// # Examples
/// ```
/// use rust_decimal::Decimal;
/// use cgt_format::format_currency;
///
/// assert_eq!(format_currency(Decimal::from(1234)), "£1,234.00");
/// assert_eq!(format_currency(Decimal::from(-100)), "-£100.00");
/// ```
pub fn format_currency(value: Decimal) -> String {
    format_currency_with_minor_units(value, '£', 2)
}

/// Format a decimal value as currency using the provided symbol and minor units.
pub fn format_currency_with_minor_units(value: Decimal, symbol: char, minor_units: u32) -> String {
    let rounded = value.round_dp_with_strategy(minor_units, RoundingStrategy::MidpointAwayFromZero);
    let abs_str = format_decimal_fixed(rounded.abs(), minor_units);
    let mut parts = abs_str.split('.');
    let integer_part = parts.next().unwrap_or("0");
    let fractional_part = parts.next();
    let formatted_int = format_with_commas_str(integer_part);

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

/// Add thousands separators to an integer string.
fn format_with_commas_str(integer_part: &str) -> String {
    let chars: Vec<char> = integer_part.chars().collect();
    let mut result = String::with_capacity(chars.len() + chars.len() / 3);
    for (i, c) in chars.iter().enumerate() {
        if i > 0 && (chars.len() - i).is_multiple_of(3) {
            result.push(',');
        }
        result.push(*c);
    }
    result
}

/// Format a decimal value to a fixed number of fractional digits.
pub fn format_decimal_fixed(value: Decimal, precision: u32) -> String {
    let rounded = value.round_dp_with_strategy(precision, RoundingStrategy::MidpointAwayFromZero);
    format!("{rounded:.precision$}", precision = precision as usize)
}

/// Format a decimal value, removing trailing zeros after the decimal point.
///
/// # Examples
/// ```
/// use rust_decimal::Decimal;
/// use cgt_format::format_decimal;
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

#[cfg(test)]
mod tests {
    use super::*;
    use cgt_money::Currency;

    #[test]
    fn test_format_currency_positive() {
        assert_eq!(format_currency(Decimal::from(100)), "£100.00");
        assert_eq!(format_currency(Decimal::from(1234)), "£1,234.00");
        assert_eq!(format_currency(Decimal::from(1000000)), "£1,000,000.00");
    }

    #[test]
    fn test_format_currency_negative() {
        assert_eq!(format_currency(Decimal::from(-20)), "-£20.00");
        assert_eq!(format_currency(Decimal::from(-1234)), "-£1,234.00");
        assert_eq!(format_currency(Decimal::new(-196, 1)), "-£19.60");
    }

    #[test]
    fn test_format_currency_zero() {
        assert_eq!(format_currency(Decimal::ZERO), "£0.00");
    }

    #[test]
    fn test_format_currency_rounds_decimals() {
        assert_eq!(format_currency(Decimal::new(10099, 2)), "£100.99");
        assert_eq!(format_currency(Decimal::new(100999, 3)), "£101.00");
        assert_eq!(format_currency(Decimal::new(-100999, 3)), "-£101.00");
    }

    #[test]
    fn test_format_decimal_fixed() {
        assert_eq!(format_decimal_fixed(Decimal::new(1234, 2), 2), "12.34");
        assert_eq!(format_decimal_fixed(Decimal::new(1234, 2), 4), "12.3400");
        assert_eq!(format_decimal_fixed(Decimal::new(-56789, 3), 2), "-56.79");
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

    #[test]
    fn test_currency_formatter_format_amount_gbp() {
        let formatter = CurrencyFormatter::uk();
        let amount = CurrencyAmount::new(Decimal::new(12345, 2), Currency::GBP);
        assert_eq!(formatter.format_amount(&amount), "£123.45");
    }

    #[test]
    fn test_currency_formatter_format_unit_gbp() {
        let formatter = CurrencyFormatter::uk();
        let amount = CurrencyAmount::new(Decimal::new(46702, 4), Currency::GBP);
        assert_eq!(formatter.format_unit(&amount), "£4.6702");
    }

    #[test]
    fn test_currency_formatter_format_unit_trims_zeros() {
        let formatter = CurrencyFormatter::uk();
        let amount = CurrencyAmount::new(Decimal::new(12500, 2), Currency::GBP);
        assert_eq!(formatter.format_unit(&amount), "£125");
    }
}
