//! FX rate lookup backed by the `hmrc-rates` crate.
//!
//! This is the only module aware of `hmrc_rates`. It isolates the dependency
//! and maps its lookup failures to the crate's own [`FxConversionError`].

use crate::money::amount::FxConversionError;
use chrono::{Datelike, NaiveDate};
use iso_currency::Currency;
use rust_decimal::Decimal;

/// HMRC monthly FX rates for converting foreign amounts to GBP.
///
/// Production instances wrap the bundled dataset compiled into the
/// `hmrc-rates` crate. Rates are keyed by currency and calendar month; lookups
/// are strict (no silent fallback to an earlier month).
#[derive(Debug, Clone)]
pub struct FxRates(Backend);

#[derive(Debug, Clone)]
enum Backend {
    /// The full HMRC monthly history compiled into `hmrc-rates`.
    Bundled(hmrc_rates::Rates),
    /// In-memory rates for tests, keyed by (ISO code, year, month), holding
    /// currency units per GBP.
    #[cfg(test)]
    Fixed(std::collections::HashMap<(String, i32, u32), Decimal>),
}

impl FxRates {
    /// The bundled HMRC monthly rates. Infallible and effectively free (the
    /// tables live in the binary's read-only data).
    pub fn bundled() -> Self {
        Self(Backend::Bundled(hmrc_rates::Rates::new()))
    }

    /// Convert `amount`, expressed in `currency`, to GBP using the HMRC rate
    /// for the month of `date`. A missing currency or month is a
    /// [`FxConversionError::MissingRate`].
    pub(crate) fn to_gbp(
        &self,
        amount: Decimal,
        currency: Currency,
        date: NaiveDate,
    ) -> Result<Decimal, FxConversionError> {
        match &self.0 {
            // `hmrc_rates` accepts a `NaiveDate` directly and divides
            // `amount / units_per_gbp`, matching HMRC's units-per-GBP figures.
            Backend::Bundled(rates) => rates
                .monthly_rate(currency.code(), date)
                .map(|rate| rate.to_gbp(amount))
                .map_err(|_| missing_rate(currency, date)),
            #[cfg(test)]
            Backend::Fixed(map) => map
                .get(&(currency.code().to_string(), date.year(), date.month()))
                .map(|units_per_gbp| amount / units_per_gbp)
                .ok_or_else(|| missing_rate(currency, date)),
        }
    }
}

#[cfg(test)]
impl FxRates {
    /// Build an in-memory rate table for tests. Each entry is
    /// `(currency, year, month, units_per_gbp)`.
    pub fn fixed(entries: impl IntoIterator<Item = (Currency, i32, u32, Decimal)>) -> Self {
        let map = entries
            .into_iter()
            .map(|(currency, year, month, units_per_gbp)| {
                ((currency.code().to_string(), year, month), units_per_gbp)
            })
            .collect();
        Self(Backend::Fixed(map))
    }
}

fn missing_rate(currency: Currency, date: NaiveDate) -> FxConversionError {
    FxConversionError::MissingRate {
        currency: currency.code().to_string(),
        year: date.year(),
        month: date.month(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal_macros::dec;

    #[test]
    fn bundled_converts_known_usd_month() {
        let rates = FxRates::bundled();
        let date = NaiveDate::from_ymd_opt(2024, 3, 15);
        assert!(date.is_some());
        let Some(date) = date else {
            return;
        };
        let result = rates.to_gbp(dec!(100), Currency::USD, date);
        assert_eq!(result, Ok(dec!(100) / dec!(1.2614)));
    }

    #[test]
    fn bundled_missing_month_is_missing_rate_error() {
        let rates = FxRates::bundled();
        let date = NaiveDate::from_ymd_opt(2099, 1, 15);
        assert!(date.is_some());
        let Some(date) = date else {
            return;
        };
        let result = rates.to_gbp(dec!(100), Currency::USD, date);
        assert!(matches!(
            result,
            Err(FxConversionError::MissingRate {
                year: 2099,
                month: 1,
                ..
            })
        ));
    }

    #[test]
    fn fixed_table_uses_supplied_rate() {
        let rates = FxRates::fixed([(Currency::USD, 2024, 3, dec!(1.25))]);
        let date = NaiveDate::from_ymd_opt(2024, 3, 15);
        assert!(date.is_some());
        let Some(date) = date else {
            return;
        };
        assert_eq!(rates.to_gbp(dec!(150), Currency::USD, date), Ok(dec!(120)));
    }
}
