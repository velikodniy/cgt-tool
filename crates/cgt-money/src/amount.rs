//! Currency amount type for monetary values with currency information.

use crate::cache::FxCache;
use chrono::{Datelike, NaiveDate};
use iso_currency::Currency;
use rust_decimal::Decimal;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use thiserror::Error;

/// Error during FX conversion to GBP.
#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum FxConversionError {
    #[error("Missing FX rate for {currency} in {year}-{month:02}")]
    MissingRate {
        currency: String,
        year: i32,
        month: u32,
    },
}

/// A monetary amount with currency information.
///
/// Stores the original amount and currency. GBP equivalents are computed on-demand
/// by consumers using FX rates.
#[derive(Debug, Clone, PartialEq)]
pub struct CurrencyAmount {
    /// The original amount as entered
    pub amount: Decimal,
    /// The currency (defaults to GBP)
    pub currency: Currency,
}

impl CurrencyAmount {
    /// Create a new CurrencyAmount with the given currency.
    pub fn new(amount: Decimal, currency: Currency) -> Self {
        Self { amount, currency }
    }

    /// Check if this amount is in GBP.
    pub fn is_gbp(&self) -> bool {
        self.currency == Currency::GBP
    }

    /// Convert this amount to GBP using FX rates for the given date.
    pub fn to_gbp(
        &self,
        date: NaiveDate,
        fx_cache: &FxCache,
    ) -> Result<Decimal, FxConversionError> {
        if self.is_gbp() {
            return Ok(self.amount);
        }

        let code = self.currency.code();
        let rate_entry = fx_cache.get(code, date.year(), date.month()).ok_or(
            FxConversionError::MissingRate {
                currency: code.to_string(),
                year: date.year(),
                month: date.month(),
            },
        )?;

        Ok(self.amount / rate_entry.rate_per_gbp)
    }

    /// Get the currency's minor units (decimal places for display).
    pub fn minor_units(&self) -> u16 {
        self.currency.exponent().unwrap_or(2)
    }

    /// Get the currency symbol.
    pub fn symbol(&self) -> String {
        self.currency.symbol().to_string()
    }

    /// Get the currency code (e.g., "USD", "EUR").
    pub fn code(&self) -> &'static str {
        self.currency.code()
    }
}

impl Default for CurrencyAmount {
    fn default() -> Self {
        Self::new(Decimal::ZERO, Currency::GBP)
    }
}

// Custom serialization - always serialize as object for consistency
impl Serialize for CurrencyAmount {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;

        let mut state = serializer.serialize_struct("CurrencyAmount", 2)?;
        state.serialize_field("amount", &self.amount)?;
        state.serialize_field("currency", self.currency.code())?;
        state.end()
    }
}

impl<'de> Deserialize<'de> for CurrencyAmount {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        use serde::de::{MapAccess, Visitor};

        struct CurrencyAmountVisitor;

        impl<'de> Visitor<'de> for CurrencyAmountVisitor {
            type Value = CurrencyAmount;

            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str("a decimal number or a currency amount object")
            }

            fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                let amount: Decimal = v.parse().map_err(E::custom)?;
                Ok(CurrencyAmount::new(amount, Currency::GBP))
            }

            fn visit_i64<E>(self, v: i64) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                Ok(CurrencyAmount::new(Decimal::from(v), Currency::GBP))
            }

            fn visit_u64<E>(self, v: u64) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                Ok(CurrencyAmount::new(Decimal::from(v), Currency::GBP))
            }

            fn visit_f64<E>(self, v: f64) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                use std::str::FromStr;
                let amount = Decimal::from_str(&v.to_string()).map_err(E::custom)?;
                Ok(CurrencyAmount::new(amount, Currency::GBP))
            }

            fn visit_map<M>(self, mut map: M) -> Result<Self::Value, M::Error>
            where
                M: MapAccess<'de>,
            {
                let mut amount: Option<Decimal> = None;
                let mut currency: Option<String> = None;

                while let Some(key) = map.next_key::<String>()? {
                    match key.as_str() {
                        "amount" => amount = Some(map.next_value()?),
                        "currency" => currency = Some(map.next_value()?),
                        // Accept legacy field but ignore it
                        "gbp" => {
                            let _: serde::de::IgnoredAny = map.next_value()?;
                        }
                        _ => {
                            let _: serde::de::IgnoredAny = map.next_value()?;
                        }
                    }
                }

                let amount = amount.ok_or_else(|| serde::de::Error::missing_field("amount"))?;
                let currency_code = currency.unwrap_or_else(|| "GBP".to_string());

                let currency = Currency::from_code(&currency_code).ok_or_else(|| {
                    serde::de::Error::custom(format!("invalid currency code: '{currency_code}'"))
                })?;

                Ok(CurrencyAmount::new(amount, currency))
            }
        }

        deserializer.deserialize_any(CurrencyAmountVisitor)
    }
}

impl JsonSchema for CurrencyAmount {
    fn schema_name() -> String {
        "CurrencyAmount".to_owned()
    }

    fn json_schema(generator: &mut schemars::r#gen::SchemaGenerator) -> schemars::schema::Schema {
        use schemars::schema::{
            InstanceType, Metadata, ObjectValidation, Schema, SchemaObject, SingleOrVec,
        };

        // CurrencyAmount can be either a plain decimal (for GBP) or an object
        let decimal_schema = generator.subschema_for::<Decimal>();

        let obj_schema = SchemaObject {
            instance_type: Some(SingleOrVec::Single(Box::new(InstanceType::Object))),
            object: Some(Box::new(ObjectValidation {
                properties: [
                    ("amount".to_string(), generator.subschema_for::<Decimal>()),
                    ("currency".to_string(), generator.subschema_for::<String>()),
                ]
                .into_iter()
                .collect(),
                required: ["amount".to_string()].into_iter().collect(),
                ..Default::default()
            })),
            metadata: Some(Box::new(Metadata {
                description: Some(
                    "A monetary amount: plain number for GBP, or object with currency".to_owned(),
                ),
                ..Default::default()
            })),
            ..Default::default()
        };

        Schema::Object(SchemaObject {
            subschemas: Some(Box::new(schemars::schema::SubschemaValidation {
                any_of: Some(vec![decimal_schema, Schema::Object(obj_schema)]),
                ..Default::default()
            })),
            ..Default::default()
        })
    }
}
