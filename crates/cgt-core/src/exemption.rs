//! UK Capital Gains Tax annual exemption amounts.
//!
//! This module provides the annual CGT exemption (Annual Exempt Amount) for each tax year.
//! The exemption is the amount of gains you can make before paying CGT.

use crate::CgtError;
use crate::config::Config;
use rust_decimal::Decimal;
use std::sync::OnceLock;

/// Global configuration loaded once with override support.
static CONFIG: OnceLock<Config> = OnceLock::new();

/// Get the global configuration, loading it with overrides on first access.
fn get_config() -> &'static Config {
    CONFIG.get_or_init(Config::load_with_overrides)
}

/// Get the UK annual CGT exemption for a given tax year start.
///
/// Uses configuration with override support. On first call, loads configuration from:
/// 1. Embedded defaults
/// 2. `./config.toml` (current directory)
/// 3. `~/.config/cgt-tool/config.toml` (user config directory)
///
/// Override files take precedence over embedded defaults.
///
/// # Arguments
/// * `year` - The calendar year when the tax year starts (e.g., 2023 for 2023/24)
///
/// # Returns
/// * `Ok(Decimal)` - The exemption amount for that tax year
/// * `Err(CgtError::UnsupportedExemptionYear)` - If the year is not supported
///
/// # Examples
/// ```
/// use cgt_core::get_exemption;
///
/// let exemption = get_exemption(2023).unwrap();
/// assert_eq!(exemption, rust_decimal::Decimal::from(6000));
/// ```
pub fn get_exemption(year: u16) -> Result<Decimal, CgtError> {
    get_config().get_exemption(year)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Test all known exemption years in a single parameterized test.
    /// The exemption values are from HMRC guidance.
    #[test]
    fn test_exemption_known_years() {
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
            let result = get_exemption(year);
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
        let result = get_exemption(2010);
        assert!(result.is_err());
        assert!(matches!(
            result,
            Err(CgtError::UnsupportedExemptionYear(2010))
        ));
    }

    #[test]
    fn test_exemption_unsupported_future() {
        let result = get_exemption(2030);
        assert!(result.is_err());
        assert!(matches!(
            result,
            Err(CgtError::UnsupportedExemptionYear(2030))
        ));
    }
}
