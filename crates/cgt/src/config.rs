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
struct RawConfig {
    #[serde(default)]
    exemptions: HashMap<String, Decimal>,
}

/// CGT tool configuration.
#[derive(Debug, Clone, Default)]
pub struct Config {
    /// Tax exemption amounts by year.
    pub exemptions: HashMap<u16, Decimal>,
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
    fn from_toml(content: &str) -> Result<Self, toml::de::Error> {
        let raw: RawConfig = toml::from_str(content)?;
        let exemptions = raw
            .exemptions
            .into_iter()
            .filter_map(|(k, v)| k.parse::<u16>().ok().map(|year| (year, v)))
            .collect();
        Ok(Self { exemptions })
    }

    /// Merge exemption overrides from a TOML string (later calls win).
    ///
    /// # Errors
    /// Returns `CgtError::ConfigError` if the TOML string cannot be parsed.
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
