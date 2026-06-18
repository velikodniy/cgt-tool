//! Shared formatting helpers for currency, dates, and numbers.
//!
//! All formatting uses UK conventions (£ symbol, DD/MM/YYYY dates, thousands
//! separators) and rounds money 2dp midpoint-away-from-zero, the single
//! rounding policy. Load-bearing for output equivalence.

pub mod plain;

use chrono::NaiveDate;
use rust_decimal::{Decimal, RoundingStrategy};

use crate::model::TaxPeriod;
use crate::money::CurrencyAmount;

/// Format a [`CurrencyAmount`] rounded to its minor units. GBP shows the £
/// symbol; other currencies show the value with the currency code.
pub fn format_currency_amount(amount: &CurrencyAmount) -> String {
    if amount.is_gbp() {
        format_gbp(amount.amount)
    } else {
        let value = format_decimal_with_precision(amount.amount, amount.minor_units() as u32);
        format!("{} {}", value, amount.code())
    }
}

/// Format a [`CurrencyAmount`] as a unit price with full precision (trailing
/// zeros trimmed).
pub fn format_price(amount: &CurrencyAmount) -> String {
    let symbol = amount.symbol();
    let value = format_decimal_trimmed(amount.amount);
    if symbol.is_empty() {
        format!("{}{}", amount.code(), value)
    } else {
        format!("{}{}", symbol, value)
    }
}

/// Format a decimal value as GBP with thousands separators and 2dp. Negative
/// values render as `-£100.00` (sign before symbol).
pub fn format_gbp(value: Decimal) -> String {
    format_with_symbol_and_precision(value, '£', 2)
}

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

/// Format a decimal with a fixed number of decimal places, rounding midpoint
/// away from zero.
pub fn format_decimal_with_precision(value: Decimal, precision: u32) -> String {
    let rounded = value.round_dp_with_strategy(precision, RoundingStrategy::MidpointAwayFromZero);
    format!("{rounded:.precision$}", precision = precision as usize)
}

/// Round a decimal to 2dp midpoint-away-from-zero, consistent with
/// [`format_gbp`]. Use when a rounded value is needed without the £ symbol.
pub fn round_gbp(value: Decimal) -> Decimal {
    value.round_dp_with_strategy(2, RoundingStrategy::MidpointAwayFromZero)
}

/// Format a decimal, removing trailing zeros and a trailing decimal point.
pub fn format_decimal_trimmed(value: Decimal) -> String {
    let s = value.to_string();
    if s.contains('.') {
        s.trim_end_matches('0').trim_end_matches('.').to_string()
    } else {
        s
    }
}

/// Format a date as DD/MM/YYYY (UK convention).
pub fn format_date(date: NaiveDate) -> String {
    date.format("%d/%m/%Y").to_string()
}

/// Format a UK tax year as "YYYY/YY".
pub fn format_tax_year(start_year: u16) -> String {
    TaxPeriod::new(start_year)
        .map(|period| period.to_string())
        .unwrap_or_else(|_| format!("{}/{:02}", start_year, (start_year + 1) % 100))
}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal_macros::dec;

    #[test]
    fn gbp_uses_thousands_and_sign_before_symbol() {
        assert_eq!(format_gbp(dec!(1234)), "£1,234.00");
        assert_eq!(format_gbp(dec!(-100)), "-£100.00");
        assert_eq!(format_gbp(dec!(1000000)), "£1,000,000.00");
    }

    #[test]
    fn gbp_midpoints_round_away_from_zero() {
        // .xx5 rounds away from zero, not to even.
        assert_eq!(format_gbp(dec!(0.125)), "£0.13");
        assert_eq!(format_gbp(dec!(-0.125)), "-£0.13");
    }

    #[test]
    fn decimal_trimmed_drops_trailing_zeros_and_point() {
        assert_eq!(format_decimal_trimmed(dec!(123.4)), "123.4");
        assert_eq!(format_decimal_trimmed(dec!(123.00)), "123");
        assert_eq!(
            format_decimal_trimmed(dec!(86.68242710796)),
            "86.68242710796"
        );
    }

    #[test]
    fn date_and_tax_year_use_uk_conventions() {
        let date = NaiveDate::from_ymd_opt(2018, 8, 28).expect("valid date");
        assert_eq!(format_date(date), "28/08/2018");
        assert_eq!(format_tax_year(2023), "2023/24");
        assert_eq!(format_tax_year(2014), "2014/15");
    }

    #[test]
    fn round_gbp_matches_format_gbp_policy() {
        assert_eq!(round_gbp(dec!(12.345)), dec!(12.35));
        assert_eq!(round_gbp(dec!(12.35)), dec!(12.35));
    }
}
