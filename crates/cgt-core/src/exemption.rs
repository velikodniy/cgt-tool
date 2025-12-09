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

    #[test]
    fn test_exemption_2014() {
        assert_eq!(get_exemption(2014).expect("2014"), Decimal::from(11000));
    }

    #[test]
    fn test_exemption_2015() {
        assert_eq!(get_exemption(2015).expect("2015"), Decimal::from(11100));
    }

    #[test]
    fn test_exemption_2016() {
        assert_eq!(get_exemption(2016).expect("2016"), Decimal::from(11100));
    }

    #[test]
    fn test_exemption_2017() {
        assert_eq!(get_exemption(2017).expect("2017"), Decimal::from(11300));
    }

    #[test]
    fn test_exemption_2018() {
        assert_eq!(get_exemption(2018).expect("2018"), Decimal::from(11700));
    }

    #[test]
    fn test_exemption_2019() {
        assert_eq!(get_exemption(2019).expect("2019"), Decimal::from(12000));
    }

    #[test]
    fn test_exemption_2020() {
        assert_eq!(get_exemption(2020).expect("2020"), Decimal::from(12300));
    }

    #[test]
    fn test_exemption_2021() {
        assert_eq!(get_exemption(2021).expect("2021"), Decimal::from(12300));
    }

    #[test]
    fn test_exemption_2022() {
        assert_eq!(get_exemption(2022).expect("2022"), Decimal::from(12300));
    }

    #[test]
    fn test_exemption_2023() {
        assert_eq!(get_exemption(2023).expect("2023"), Decimal::from(6000));
    }

    #[test]
    fn test_exemption_2024() {
        assert_eq!(get_exemption(2024).expect("2024"), Decimal::from(3000));
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
