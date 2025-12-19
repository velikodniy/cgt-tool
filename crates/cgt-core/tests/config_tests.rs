//! Tests for cgt-core config.rs (Config loading and exemptions)

#![allow(clippy::expect_used)]

use cgt_core::Config;
use rust_decimal::Decimal;

#[test]
fn test_embedded_config_loads() {
    let config = Config::embedded();
    assert!(!config.exemptions.is_empty());
}

#[test]
fn test_embedded_has_2023_exemption() {
    let config = Config::embedded();
    assert_eq!(config.get_exemption(2023).ok(), Some(Decimal::from(6000)));
}

#[test]
fn test_embedded_has_all_years() {
    let config = Config::embedded();
    for year in 2014..=2024 {
        assert!(
            config.get_exemption(year).is_ok(),
            "Missing exemption for year {year}"
        );
    }
}

#[test]
fn test_unsupported_year_returns_error() {
    let config = Config::embedded();
    assert!(config.get_exemption(2010).is_err());
    assert!(config.get_exemption(2030).is_err());
}

#[test]
fn test_load_with_overrides_includes_embedded() {
    let config = Config::load_with_overrides();
    // Should still have embedded values even if no override files exist
    assert!(config.get_exemption(2023).is_ok());
}
