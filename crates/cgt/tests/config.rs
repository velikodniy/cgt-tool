//! Tests for cgt config.rs (Config loading and exemptions)

#![allow(clippy::expect_used)]

use cgt::Config;
use rust_decimal::Decimal;

#[test]
fn test_embedded_config_loads() {
    let config = Config::embedded().expect("embedded config should load");
    assert!(!config.exemptions.is_empty());
}

#[test]
fn test_embedded_has_2023_exemption() {
    let config = Config::embedded().expect("embedded config should load");
    assert_eq!(config.get_exemption(2023).ok(), Some(Decimal::from(6000)));
}

#[test]
fn test_embedded_has_all_years() {
    let config = Config::embedded().expect("embedded config should load");
    for year in 2014..=2026 {
        assert!(
            config.get_exemption(year).is_ok(),
            "Missing exemption for year {year}"
        );
    }
}

#[test]
fn test_unsupported_year_returns_error() {
    let config = Config::embedded().expect("embedded config should load");
    assert!(config.get_exemption(2010).is_err());
    assert!(config.get_exemption(2030).is_err());
}

#[test]
fn test_apply_overrides_keeps_embedded_values() {
    let mut config = Config::embedded().expect("embedded config should load");
    config
        .apply_overrides_toml("[exemptions]\n\"2030\" = 5000\n")
        .expect("override should parse");
    // Should still have embedded values after applying overrides
    assert!(config.get_exemption(2023).is_ok());
    assert_eq!(config.get_exemption(2030).ok(), Some(Decimal::from(5000)));
}

#[test]
fn test_apply_overrides_takes_precedence() {
    let mut config = Config::embedded().expect("embedded config should load");
    config
        .apply_overrides_toml("[exemptions]\n\"2023\" = 9999\n")
        .expect("override should parse");
    // Override value wins over the embedded value
    assert_eq!(config.get_exemption(2023).ok(), Some(Decimal::from(9999)));
}

#[test]
fn test_apply_overrides_later_calls_win() {
    let mut config = Config::embedded().expect("embedded config should load");
    config
        .apply_overrides_toml("[exemptions]\n\"2023\" = 7000\n")
        .expect("first override should parse");
    config
        .apply_overrides_toml("[exemptions]\n\"2023\" = 8000\n")
        .expect("second override should parse");
    // The later override wins
    assert_eq!(config.get_exemption(2023).ok(), Some(Decimal::from(8000)));
}

#[test]
fn test_apply_overrides_invalid_toml_is_error() {
    let mut config = Config::embedded().expect("embedded config should load");
    let result = config.apply_overrides_toml("not valid toml [");
    assert!(result.is_err());
    // The config keeps its embedded values
    assert!(config.get_exemption(2023).is_ok());
}
