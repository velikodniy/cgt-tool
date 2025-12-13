use iso_currency::Currency;
use rust_decimal::Decimal;
use std::path::PathBuf;
use std::time::SystemTime;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct RateKey {
    pub code: Currency,
    pub year: i32,
    pub month: u32,
}

impl RateKey {
    pub fn new(code: Currency, year: i32, month: u32) -> Self {
        Self { code, year, month }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RateSource {
    Bundled {
        period: Option<String>,
    },
    Folder {
        path: PathBuf,
        period: Option<String>,
        modified: Option<SystemTime>,
    },
}

#[derive(Debug, Clone, PartialEq)]
pub struct RateEntry {
    pub key: RateKey,
    /// HMRC exchange rate: foreign currency units per 1 GBP.
    /// To convert foreign amount to GBP: `foreign_amount / rate_per_gbp`
    pub rate_per_gbp: Decimal,
    pub source: RateSource,
    pub minor_units: u8,
    pub symbol: Option<String>,
}
