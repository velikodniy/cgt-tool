use crate::types::{RateEntry, RateKey};
use iso_currency::Currency;
use std::collections::HashMap;

#[derive(Debug, Default, Clone)]
pub struct FxCache {
    rates: HashMap<RateKey, RateEntry>,
}

impl FxCache {
    pub fn new() -> Self {
        Self {
            rates: HashMap::new(),
        }
    }

    pub fn insert(&mut self, entry: RateEntry) {
        self.rates.insert(entry.key.clone(), entry);
    }

    pub fn extend(&mut self, entries: Vec<RateEntry>) {
        for entry in entries {
            self.insert(entry);
        }
    }

    pub fn get(&self, currency: Currency, year: i32, month: u32) -> Option<&RateEntry> {
        let key = RateKey {
            code: currency,
            year,
            month,
        };
        self.rates.get(&key)
    }

    pub fn len(&self) -> usize {
        self.rates.len()
    }

    pub fn is_empty(&self) -> bool {
        self.rates.is_empty()
    }

    /// Check whether any rate exists for the given currency code across all cached periods.
    pub fn has_currency(&self, code: &str) -> bool {
        let code = code.trim().to_uppercase();
        let Some(currency) = Currency::from_code(&code) else {
            return false;
        };
        self.rates.keys().any(|k| k.code == currency)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::RateSource;
    use rust_decimal::Decimal;

    fn make_entry(code: &str, year: i32, month: u32) -> RateEntry {
        let currency = Currency::from_code(code).expect("valid currency code in test");
        RateEntry {
            key: RateKey::new(currency, year, month),
            rate_per_gbp: Decimal::new(125, 2),
            source: RateSource::Bundled { period: None },
            minor_units: 2,
            symbol: None,
        }
    }

    #[test]
    fn has_currency_returns_true_for_present_currency() {
        let mut cache = FxCache::new();
        cache.insert(make_entry("USD", 2024, 1));
        assert!(cache.has_currency("USD"));
    }

    #[test]
    fn has_currency_returns_false_for_absent_currency() {
        let mut cache = FxCache::new();
        cache.insert(make_entry("USD", 2024, 1));
        assert!(!cache.has_currency("EUR"));
    }

    #[test]
    fn has_currency_is_case_insensitive() {
        let mut cache = FxCache::new();
        cache.insert(make_entry("USD", 2024, 1));
        assert!(cache.has_currency("usd"));
        assert!(cache.has_currency("Usd"));
    }

    #[test]
    fn has_currency_returns_false_for_invalid_iso_code() {
        let mut cache = FxCache::new();
        cache.insert(make_entry("USD", 2024, 1));
        assert!(!cache.has_currency("XYZ123"));
        assert!(!cache.has_currency(""));
    }

    #[test]
    fn has_currency_returns_false_on_empty_cache() {
        let cache = FxCache::new();
        assert!(!cache.has_currency("USD"));
    }
}
