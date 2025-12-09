//! Configuration management for CGT tool.
//!
//! This module provides configuration loading with embedded defaults
//! and optional override files.

use crate::CgtError;
use rust_decimal::Decimal;
use serde::Deserialize;
use std::collections::HashMap;
use std::path::PathBuf;

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
    pub fn embedded() -> Self {
        Self::from_toml(EMBEDDED_CONFIG).unwrap_or_else(|e| {
            eprintln!("Warning: Failed to parse embedded config: {e}");
            Self {
                exemptions: HashMap::new(),
            }
        })
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

    /// Load configuration with override support.
    ///
    /// Checks for override files in the following order:
    /// 1. `./config.toml` (current directory)
    /// 2. `~/.config/cgt-tool/config.toml` (user config directory)
    ///
    /// Override files are merged with embedded defaults. Values from
    /// override files take precedence.
    pub fn load_with_overrides() -> Self {
        let mut config = Self::embedded();

        // Try loading override files
        let override_paths = Self::override_paths();
        for path in override_paths {
            if path.exists()
                && let Ok(content) = std::fs::read_to_string(&path)
                && let Ok(override_config) = Self::from_toml(&content)
            {
                // Merge exemptions (override takes precedence)
                config.exemptions.extend(override_config.exemptions);
            }
        }

        config
    }

    /// Get potential override file paths.
    fn override_paths() -> Vec<PathBuf> {
        let mut paths = Vec::new();

        // Current directory
        paths.push(PathBuf::from("config.toml"));

        // User config directory
        if let Some(home) = dirs_home() {
            paths.push(home.join(".config").join("cgt-tool").join("config.toml"));
        }

        paths
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

/// Get the user's home directory without external dependencies.
fn dirs_home() -> Option<PathBuf> {
    std::env::var_os("HOME").map(PathBuf::from)
}

#[cfg(test)]
mod tests {
    use super::*;

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
}
