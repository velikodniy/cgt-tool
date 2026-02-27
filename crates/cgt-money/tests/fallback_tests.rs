//! Tests for FX rate fallback behavior between provided folder and bundled rates.

use cgt_money::{Currency, FxCache, RateFile, load_cache_with_overrides, load_default_cache};
use std::path::PathBuf;
use std::time::{Duration, SystemTime};

const FOLDER_XML: &str = r#"<?xml version="1.0" encoding="UTF-8"?>
<exchangeRateMonthList Period="01/Jan/2025 to 31/Jan/2025">
  <exchangeRate>
    <countryName>Eurozone</countryName>
    <countryCode>EU</countryCode>
    <currencyName>Euro</currencyName>
    <currencyCode>EUR</currencyCode>
    <rateNew>1.2500</rateNew>
  </exchangeRate>
</exchangeRateMonthList>
"#;

#[test]
fn folder_rate_overrides_bundled_for_same_month() {
    // Create folder XML for Dec 2024 (same month as bundled)
    let folder_xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<exchangeRateMonthList Period="01/Dec/2024 to 31/Dec/2024">
  <exchangeRate>
    <countryName>Eurozone</countryName>
    <countryCode>EU</countryCode>
    <currencyName>Euro</currencyName>
    <currencyCode>EUR</currencyCode>
    <rateNew>9.9999</rateNew>
  </exchangeRate>
</exchangeRateMonthList>
"#;

    let cache = load_cache_with_overrides(vec![RateFile {
        name: PathBuf::from("2024-12.xml"),
        modified: Some(SystemTime::UNIX_EPOCH + Duration::from_secs(1)),
        xml: folder_xml.to_string(),
    }])
    .unwrap();

    // Folder rate should override bundled for EUR Dec 2024
    let eur = cache.get(Currency::EUR, 2024, 12).unwrap();
    assert_eq!(
        eur.rate_per_gbp.to_string(),
        "9.9999",
        "Folder rate should override bundled"
    );

    // USD from bundled should still be present (not overridden)
    let usd = cache.get(Currency::USD, 2024, 12).unwrap();
    assert!(
        usd.rate_per_gbp.to_string() != "9.9999",
        "USD should use bundled rate"
    );
}

#[test]
fn folder_adds_rates_for_missing_months() {
    let cache = load_cache_with_overrides(vec![RateFile {
        name: PathBuf::from("2025-01.xml"),
        modified: Some(SystemTime::UNIX_EPOCH + Duration::from_secs(2)),
        xml: FOLDER_XML.to_string(),
    }])
    .unwrap();

    // Jan 2025 EUR should come from folder (bundled covers 2015-2025)
    let eur = cache.get(Currency::EUR, 2025, 1).unwrap();
    assert_eq!(
        eur.rate_per_gbp.to_string(),
        "1.2500",
        "Folder rate for Jan 2025 should be available"
    );

    // Dec 2024 EUR should still come from bundled
    let eur_dec = cache.get(Currency::EUR, 2024, 12).unwrap();
    assert!(
        eur_dec.rate_per_gbp.to_string() != "1.2500",
        "Bundled Dec 2024 rate should remain"
    );
}

#[test]
fn missing_rate_returns_none() {
    let cache = load_default_cache().unwrap();

    // Far future month should not exist
    assert!(
        cache.get(Currency::EUR, 2130, 1).is_none(),
        "Rate for far future month should return None"
    );

    // Far future month should not exist
    assert!(
        cache.get(Currency::USD, 2130, 1).is_none(),
        "Rate for far future should return None"
    );
}

#[test]
fn empty_folder_uses_bundled_only() {
    let cache = load_cache_with_overrides(Vec::new()).unwrap();

    // Should still have bundled rates
    assert!(
        cache.get(Currency::USD, 2024, 12).is_some(),
        "Bundled rates should be available with empty folder"
    );
}

#[test]
fn cache_lookup_with_currency_enum() {
    let cache = load_default_cache().unwrap();

    // Currency enum guarantees a valid, correctly-cased code
    let result = cache.get(Currency::USD, 2024, 12);
    assert!(result.is_some(), "Currency enum lookup should work");
}

#[test]
fn bundled_rates_contain_major_currencies() {
    let cache = load_default_cache().unwrap();

    // These common currencies should be present in the bundled rates
    let major_currencies = [
        Currency::USD,
        Currency::EUR,
        Currency::JPY,
        Currency::GBP,
        Currency::CHF,
        Currency::CAD,
        Currency::AUD,
        Currency::CNY,
    ];

    for currency in major_currencies {
        // GBP won't be in the rates (it's the base currency)
        if currency == Currency::GBP {
            assert!(
                cache.get(currency, 2024, 12).is_none(),
                "GBP should not be in rates (it's the base)"
            );
        } else {
            assert!(
                cache.get(currency, 2024, 12).is_some(),
                "Major currency {} should be in bundled rates",
                currency.code()
            );
        }
    }
}

#[test]
fn no_folder_loads_bundled_only() {
    let cache = load_default_cache().unwrap();

    assert!(
        !cache.is_empty(),
        "Cache should not be empty with bundled rates"
    );
    assert!(
        cache.get(Currency::USD, 2024, 12).is_some(),
        "Should have USD from bundled"
    );
}

#[test]
fn cache_is_empty_returns_false_with_rates() {
    let cache = load_default_cache().unwrap();
    assert!(!cache.is_empty());

    let empty_cache = FxCache::new();
    assert!(empty_cache.is_empty());
}

#[test]
fn bundled_rates_cover_multiple_years() {
    let cache = load_default_cache().unwrap();

    // Verify rates exist for 2015-2024 (full 10-year coverage)
    for year in 2015..=2024 {
        assert!(
            cache.get(Currency::USD, year, 6).is_some(),
            "Should have USD rate for {} June",
            year
        );
        assert!(
            cache.get(Currency::EUR, year, 1).is_some(),
            "Should have EUR rate for {} January",
            year
        );
    }
}
