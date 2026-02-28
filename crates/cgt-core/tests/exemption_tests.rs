//! Tests for CGT exemption lookup via Config.

#![allow(clippy::expect_used)]

use cgt_core::{CgtError, Config};
use rust_decimal::Decimal;

/// Test all known exemption years in a single parameterized test.
/// The exemption values are from HMRC guidance.
#[test]
fn test_exemption_known_years() {
    let config = Config::embedded().expect("embedded config should load");

    let cases: &[(u16, i64)] = &[
        (2014, 11000),
        (2015, 11100),
        (2016, 11100),
        (2017, 11300),
        (2018, 11700),
        (2019, 12000),
        (2020, 12300),
        (2021, 12300),
        (2022, 12300),
        (2023, 6000),
        (2024, 3000),
    ];

    for &(year, expected) in cases {
        let result = config.get_exemption(year);
        assert!(
            result.is_ok(),
            "Year {} should have exemption but got error: {:?}",
            year,
            result.err()
        );
        assert_eq!(
            result.unwrap(),
            Decimal::from(expected),
            "Year {} should have exemption {}",
            year,
            expected
        );
    }
}

#[test]
fn test_exemption_unsupported_past() {
    let config = Config::embedded().expect("embedded config should load");
    let result = config.get_exemption(2010);
    assert!(result.is_err());
    assert!(matches!(
        result,
        Err(CgtError::UnsupportedExemptionYear(2010))
    ));
}

#[test]
fn test_exemption_unsupported_future() {
    let config = Config::embedded().expect("embedded config should load");
    let result = config.get_exemption(2030);
    assert!(result.is_err());
    assert!(matches!(
        result,
        Err(CgtError::UnsupportedExemptionYear(2030))
    ));
}
