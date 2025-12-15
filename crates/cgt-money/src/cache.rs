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

    pub fn get(&self, code: &str, year: i32, month: u32) -> Option<&RateEntry> {
        let code = code.trim().to_uppercase();
        let currency = Currency::from_code(&code)?;
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
}
