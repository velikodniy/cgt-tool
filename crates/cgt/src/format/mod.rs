//! Shared formatting helpers for currency, dates, and numbers.
//!
//! All formatting uses UK conventions (£ symbol, DD/MM/YYYY dates, thousands
//! separators) and rounds money 2dp midpoint-away-from-zero, the single
//! rounding policy. Load-bearing for output equivalence.

pub mod plain;

use chrono::NaiveDate;
use num_bigint::BigInt;
use num_traits::{Signed, Zero};
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

/// Format `total / quantity` with two decimal places, rounding once from the
/// exact ratio. A zero quantity produces `0.00`.
pub fn format_unit_cost(total: Decimal, quantity: Decimal) -> String {
    if quantity.is_zero() {
        return "0.00".into();
    }
    let numerator =
        BigInt::from(total.mantissa()) * BigInt::from(10_u8).pow(quantity.scale()) * 100_u8;
    let denominator = BigInt::from(quantity.mantissa()) * BigInt::from(10_u8).pow(total.scale());
    let negative = numerator.is_negative() != denominator.is_negative();
    let numerator = numerator.abs();
    let denominator = denominator.abs();
    let mut pennies = &numerator / &denominator;
    if (&numerator % &denominator) * 2_u8 >= denominator {
        pennies += 1_u8;
    }
    let sign = if negative && !pennies.is_zero() {
        "-"
    } else {
        ""
    };
    let whole = &pennies / 100_u8;
    let fraction = (&pennies % 100_u8).to_string();
    format!("{sign}{whole}.{fraction:0>2}")
}

/// Format the exact per-share ratio as GBP with thousands separators.
pub fn format_gbp_unit_cost(total: Decimal, quantity: Decimal) -> String {
    let value = format_unit_cost(total, quantity);
    let absolute = value.strip_prefix('-').unwrap_or(&value);
    format_money_string(absolute, value.starts_with('-'), '£')
}

fn format_with_symbol_and_precision(value: Decimal, symbol: char, minor_units: u32) -> String {
    let rounded = value.round_dp_with_strategy(minor_units, RoundingStrategy::MidpointAwayFromZero);
    let abs_str = format_decimal_with_precision(rounded.abs(), minor_units);
    format_money_string(
        &abs_str,
        rounded.is_sign_negative() && !rounded.is_zero(),
        symbol,
    )
}

fn format_money_string(absolute: &str, negative: bool, symbol: char) -> String {
    let mut parts = absolute.split('.');
    let integer_part = parts.next().unwrap_or("0");
    let fractional_part = parts.next();
    let formatted_int = add_thousands_separators(integer_part);

    let formatted = if let Some(frac) = fractional_part {
        format!("{formatted_int}.{frac}")
    } else {
        formatted_int
    };

    if negative {
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

/// The single money rounding policy: 2dp, midpoint rounded away from zero, with
/// negative zero normalized to zero so serialized/rendered money never shows
/// `-£0.00`. [`format_gbp`] and the report serializers route through this. Use
/// wherever a rounded money value is needed. Load-bearing for output
/// equivalence; `rust_decimal` is path-dependent.
pub fn round_money(value: Decimal) -> Decimal {
    let rounded = value.round_dp_with_strategy(2, RoundingStrategy::MidpointAwayFromZero);
    // Clear the sign of a negative zero (so money never renders "-£0.00")
    // while preserving the 2dp scale (a zero total still serializes as "0.00").
    if rounded.is_zero() {
        rounded.abs()
    } else {
        rounded
    }
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
        .unwrap_or_else(|_| format!("{}/{:02}", start_year, (u32::from(start_year) + 1) % 100))
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
    fn unit_cost_rounds_the_exact_ratio_once() {
        assert_eq!(
            format_unit_cost(dec!(1), dec!(200.00000000000000000000000001)),
            "0.00"
        );
        assert_eq!(format_unit_cost(dec!(10.0005), dec!(0.1)), "100.01");
        assert_eq!(format_unit_cost(dec!(0.01), dec!(2)), "0.01");
        assert_eq!(format_unit_cost(dec!(-0.01), dec!(2)), "-0.01");
        assert_eq!(format_unit_cost(dec!(0.01), dec!(-2)), "-0.01");
        assert_eq!(format_unit_cost(dec!(-0.001), dec!(1)), "0.00");
        assert_eq!(format_unit_cost(Decimal::MAX, Decimal::ZERO), "0.00");
    }

    #[test]
    fn unit_cost_preserves_large_amounts_and_currency_formatting() {
        assert_eq!(
            format_unit_cost(Decimal::MAX, dec!(0.1)),
            "792281625142643375935439503350.00"
        );
        assert_eq!(
            format_gbp_unit_cost(dec!(9007199254740992.99), dec!(1)),
            "£9,007,199,254,740,992.99"
        );
        assert_eq!(format_gbp_unit_cost(dec!(-1000.005), dec!(1)), "-£1,000.01");
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
    fn round_money_rounds_away_from_zero_and_normalizes_negative_zero() {
        assert_eq!(round_money(dec!(12.345)), dec!(12.35));
        assert_eq!(round_money(dec!(12.35)), dec!(12.35));
        // F18: a sub-penny negative must not retain the sign bit.
        assert!(!round_money(dec!(-0.001)).is_sign_negative());
        assert_eq!(format_gbp(dec!(-0.004)), "£0.00");
        assert_eq!(format_gbp(dec!(-0.001)), "£0.00");
        // A genuine penny loss still shows its sign.
        assert_eq!(format_gbp(dec!(-0.005)), "-£0.01");
    }
}
