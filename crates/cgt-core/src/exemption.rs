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
    CONFIG.get_or_init(|| Config::load_with_overrides().unwrap_or_default())
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
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// use cgt_core::get_exemption;
///
/// let exemption = get_exemption(2023)?;
/// assert_eq!(exemption, rust_decimal::Decimal::from(6000));
/// # Ok(())
/// # }
/// ```
pub fn get_exemption(year: u16) -> Result<Decimal, CgtError> {
    get_config().get_exemption(year)
}
