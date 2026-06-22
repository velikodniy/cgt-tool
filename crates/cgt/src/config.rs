//! Configuration management for CGT tool.
//!
//! This module provides configuration loading with embedded defaults
//! and optional overrides supplied as TOML strings. File discovery is
//! the responsibility of the caller (e.g. the CLI).

use crate::CgtError;
use rust_decimal::Decimal;
use serde::Deserialize;
use std::collections::HashMap;

/// Embedded default configuration.
static EMBEDDED_CONFIG: &str = include_str!("../data/config.toml");

/// Raw configuration as parsed from TOML (uses string keys).
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
struct RawConfig {
    #[serde(default)]
    exemptions: HashMap<String, Decimal>,
}

/// CGT tool configuration.
#[derive(Debug, Clone, Default)]
pub struct Config {
    /// Tax exemption amounts by year.
    pub exemptions: HashMap<u16, Decimal>,
    /// When set, a tax year with no configured exemption gets no allowance
    /// (with a warning) instead of aborting the whole report.
    pub allow_missing_exemption: bool,
}

impl Config {
    /// Load the embedded default configuration.
    ///
    /// This configuration is compiled into the binary and provides
    /// default exemption values for supported tax years.
    ///
    /// # Errors
    /// Returns `CgtError::ConfigError` if the embedded configuration cannot be parsed.
    pub fn embedded() -> Result<Self, CgtError> {
        Self::from_toml(EMBEDDED_CONFIG)
            .map_err(|e| CgtError::ConfigError(format!("failed to parse embedded config: {e}")))
    }

    /// Parse configuration from TOML string.
    ///
    /// Strict: unknown tables/fields and exemption keys that are not a
    /// tax-year start (e.g. `"203O"`) are errors, never silent no-ops.
    fn from_toml(content: &str) -> Result<Self, String> {
        let raw: RawConfig = toml::from_str(content).map_err(|e| e.to_string())?;
        let mut exemptions = HashMap::with_capacity(raw.exemptions.len());
        for (key, value) in raw.exemptions {
            let year = key
                .parse::<u16>()
                .map_err(|_| format!("invalid exemption year key: '{key}'"))?;
            exemptions.insert(year, value);
        }
        Ok(Self {
            exemptions,
            ..Self::default()
        })
    }

    /// Merge exemption overrides from a TOML string (later calls win).
    ///
    /// The expected shape mirrors the embedded config: an `[exemptions]`
    /// table mapping quoted tax-year start years to amounts in GBP.
    ///
    /// ```
    /// # fn main() -> Result<(), cgt::CgtError> {
    /// let mut config = cgt::Config::embedded()?;
    /// config.apply_overrides_toml("[exemptions]\n\"2024\" = 6000\n")?;
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// # Errors
    /// Returns `CgtError::ConfigError` if the TOML cannot be parsed, contains
    /// an unknown table, or has an exemption key that is not a valid year.
    /// On error the existing configuration is left unchanged.
    pub fn apply_overrides_toml(&mut self, toml_text: &str) -> Result<(), CgtError> {
        let overrides = Self::from_toml(toml_text)
            .map_err(|e| CgtError::ConfigError(format!("failed to parse override config: {e}")))?;
        self.exemptions.extend(overrides.exemptions);
        Ok(())
    }

    /// Get the exemption amount for a tax year.
    ///
    /// # Arguments
    /// * `year` - The calendar year when the tax year starts (e.g., 2023 for 2023/24)
    ///
    /// # Returns
    /// * `Ok(Decimal)` - The exemption amount for that tax year
    /// * `Err(CgtError::UnsupportedExemptionYear)` - If the year is not in the configuration
    pub fn get_exemption(&self, year: u16) -> Result<Decimal, CgtError> {
        self.exemptions
            .get(&year)
            .copied()
            .ok_or(CgtError::UnsupportedExemptionYear(year))
    }
}
