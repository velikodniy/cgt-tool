use cgt_money::{
    FxCache, FxParseError, RateFile, RateSource, load_cache_with_overrides, parse_monthly_rates,
};
use iso_currency::Currency;
use rust_decimal::Decimal;
use std::path::PathBuf;
use std::str::FromStr;

const SAMPLE_XML: &str = r#"<?xml version="1.0" encoding="UTF-8"?>
<exchangeRateMonthList Period="01/Mar/2025 to 31/Mar/2025">
  <exchangeRate>
    <countryName>Eurozone</countryName>
    <countryCode>EU</countryCode>
    <currencyName>Euro</currencyName>
    <currencyCode>EUR</currencyCode>
    <rateNew>1.1328</rateNew>
  </exchangeRate>
  <exchangeRate>
    <countryName>USA</countryName>
    <countryCode>US</countryCode>
    <currencyName>Dollar</currencyName>
    <currencyCode>USD</currencyCode>
    <rateNew>1.3126</rateNew>
  </exchangeRate>
</exchangeRateMonthList>
"#;

#[test]
fn parses_monthly_rates_and_enriches_currency() {
    let entries = parse_monthly_rates(
        SAMPLE_XML,
        RateSource::Bundled { period: None },
        Some((2025, 3)),
    )
    .unwrap();
    assert_eq!(entries.len(), 2);

    let eur = entries
        .iter()
        .find(|e| e.key.code == Currency::EUR)
        .unwrap();
    assert_eq!(eur.key.year, 2025);
    assert_eq!(eur.key.month, 3);
    assert_eq!(
        u16::from(eur.minor_units),
        Currency::EUR.exponent().unwrap()
    );
    assert_eq!(eur.rate_per_gbp, Decimal::from_str("1.1328").unwrap());
    assert!(eur.symbol.as_deref().unwrap().contains("€"));
}

#[test]
fn load_cache_merges_folder_over_bundled() {
    let cache = load_cache_with_overrides(vec![RateFile {
        name: PathBuf::from("2025-03.xml"),
        modified: None,
        xml: SAMPLE_XML.to_string(),
    }])
    .unwrap();

    // Folder-provided rates for March 2025 should be present
    let eur = cache.get(Currency::EUR, 2025, 3).unwrap();
    assert_eq!(eur.rate_per_gbp, Decimal::from_str_exact("1.1328").unwrap());
    // Ensure bundled rates still present
    assert!(cache.get(Currency::USD, 2024, 12).is_some());
}

#[test]
fn period_mismatch_is_rejected() {
    let err = parse_monthly_rates(
        SAMPLE_XML,
        RateSource::Bundled { period: None },
        Some((2024, 12)),
    )
    .unwrap_err();

    assert!(matches!(err, FxParseError::PeriodMismatch { .. }));
}

#[test]
fn cache_get_empty_returns_none() {
    let cache = FxCache::new();
    assert!(cache.get(Currency::EUR, 2025, 3).is_none());
}

#[test]
fn zero_rate_is_rejected() {
    let xml_with_zero_rate = r#"<?xml version="1.0" encoding="UTF-8"?>
<exchangeRateMonthList Period="01/Mar/2025 to 31/Mar/2025">
  <exchangeRate>
    <currencyCode>EUR</currencyCode>
    <rateNew>0</rateNew>
  </exchangeRate>
</exchangeRateMonthList>
"#;

    let err = parse_monthly_rates(
        xml_with_zero_rate,
        RateSource::Bundled { period: None },
        Some((2025, 3)),
    )
    .unwrap_err();

    assert!(
        matches!(err, FxParseError::NonPositiveRate { .. }),
        "Expected NonPositiveRate error for zero rate, got: {err:?}"
    );
}

#[test]
fn negative_rate_is_rejected() {
    let xml_with_negative_rate = r#"<?xml version="1.0" encoding="UTF-8"?>
<exchangeRateMonthList Period="01/Mar/2025 to 31/Mar/2025">
  <exchangeRate>
    <currencyCode>USD</currencyCode>
    <rateNew>-1.5</rateNew>
  </exchangeRate>
</exchangeRateMonthList>
"#;

    let err = parse_monthly_rates(
        xml_with_negative_rate,
        RateSource::Bundled { period: None },
        Some((2025, 3)),
    )
    .unwrap_err();

    assert!(
        matches!(err, FxParseError::NonPositiveRate { .. }),
        "Expected NonPositiveRate error for negative rate, got: {err:?}"
    );
}
