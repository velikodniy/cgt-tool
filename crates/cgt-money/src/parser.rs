use crate::types::{RateEntry, RateKey, RateSource};
use chrono::{Datelike, NaiveDate};
use iso_currency::Currency;
use quick_xml::de::from_str;
use rust_decimal::Decimal;
use serde::Deserialize;
use std::str::FromStr;

#[derive(Debug, thiserror::Error)]
pub enum FxParseError {
    #[error("XML parse error: {0}")]
    Xml(#[from] quick_xml::DeError),
    #[error("Period attribute missing or invalid: {0}")]
    InvalidPeriod(String),
    #[error(
        "Period mismatch: expected {expected_year:04}-{expected_month:02}, found {found_year:04}-{found_month:02} in {period}"
    )]
    PeriodMismatch {
        expected_year: i32,
        expected_month: u32,
        found_year: i32,
        found_month: u32,
        period: String,
    },
    #[error("Invalid rate value for {code}: {source}")]
    InvalidRate {
        code: String,
        source: String,
        #[source]
        error: rust_decimal::Error,
    },
    #[error("Non-positive FX rate for {code}: {rate} (must be > 0 to avoid divide-by-zero)")]
    NonPositiveRate { code: String, rate: Decimal },
}

/// Single-month format from trade-tariff.service.gov.uk
#[derive(Debug, Deserialize)]
struct ExchangeRateMonthList {
    #[serde(rename = "@Period")]
    period: String,
    #[serde(rename = "exchangeRate")]
    exchange_rates: Vec<ExchangeRate>,
}

#[derive(Debug, Deserialize)]
struct ExchangeRate {
    #[serde(rename = "countryName")]
    _country_name: Option<String>,
    #[serde(rename = "countryCode")]
    _country_code: Option<String>,
    #[serde(rename = "currencyName")]
    _currency_name: Option<String>,
    #[serde(rename = "currencyCode")]
    currency_code: String,
    #[serde(rename = "rateNew")]
    rate_new: String,
}

fn parse_period(period: &str) -> Result<(i32, u32), FxParseError> {
    let start = period.split_whitespace().next().unwrap_or(period);
    let date_str = start.split("to").next().unwrap_or(start).trim();
    let parsed = NaiveDate::parse_from_str(date_str, "%d/%b/%Y")
        .map_err(|_| FxParseError::InvalidPeriod(period.to_string()))?;
    Ok((parsed.year(), parsed.month()))
}

fn currency_minor_units(currency: Currency) -> u8 {
    currency.exponent().unwrap_or(2) as u8
}

fn currency_symbol(currency: Currency) -> Option<String> {
    let symbol = currency.symbol();
    let rendered = symbol.to_string();
    if rendered.is_empty() || rendered == "¤" {
        None
    } else {
        Some(rendered)
    }
}

/// Parse monthly rates XML from trade-tariff.service.gov.uk format.
/// Optionally enforce that the parsed period matches an expected (year, month).
pub fn parse_monthly_rates(
    xml: &str,
    mut source: RateSource,
    expected_year_month: Option<(i32, u32)>,
) -> Result<Vec<RateEntry>, FxParseError> {
    let parsed: ExchangeRateMonthList = from_str(xml)?;
    let (year, month) = parse_period(&parsed.period)?;

    if let Some((expected_year, expected_month)) = expected_year_month
        && (year != expected_year || month != expected_month)
    {
        return Err(FxParseError::PeriodMismatch {
            expected_year,
            expected_month,
            found_year: year,
            found_month: month,
            period: parsed.period.clone(),
        });
    }

    match &mut source {
        RateSource::Bundled { period } => *period = Some(parsed.period.clone()),
        RateSource::Folder { period, .. } => *period = Some(parsed.period.clone()),
    }

    let mut entries = Vec::with_capacity(parsed.exchange_rates.len());
    for rate in parsed.exchange_rates {
        let code_raw = rate.currency_code.trim().to_uppercase();

        // Skip currencies not recognized by iso_currency (e.g., VEF - old Venezuelan Bolívar)
        let Some(currency) = Currency::from_code(&code_raw) else {
            continue;
        };

        let rate_decimal =
            Decimal::from_str(rate.rate_new.trim()).map_err(|error| FxParseError::InvalidRate {
                code: code_raw.clone(),
                source: rate.rate_new.clone(),
                error,
            })?;

        if rate_decimal <= Decimal::ZERO {
            return Err(FxParseError::NonPositiveRate {
                code: code_raw,
                rate: rate_decimal,
            });
        }

        let key = RateKey::new(currency, year, month);
        let minor_units = currency_minor_units(currency);
        let symbol = currency_symbol(currency);

        entries.push(RateEntry {
            key,
            rate_per_gbp: rate_decimal,
            source: source.clone(),
            minor_units,
            symbol,
        });
    }

    Ok(entries)
}
