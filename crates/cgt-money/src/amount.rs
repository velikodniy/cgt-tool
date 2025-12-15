//! Currency amount type for monetary values with currency information.

use iso_currency::Currency;
use rust_decimal::Decimal;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// A monetary amount with currency information.
///
/// Stores the original amount and currency, along with the GBP equivalent
/// for UK tax calculations. For GBP amounts, `gbp` equals `amount`.
#[derive(Debug, Clone, PartialEq)]
pub struct CurrencyAmount {
    /// The original amount as entered
    pub amount: Decimal,
    /// The currency (defaults to GBP)
    pub currency: Currency,
    /// The GBP equivalent (same as amount for GBP, converted for others)
    pub gbp: Decimal,
}

impl CurrencyAmount {
    /// Create a new CurrencyAmount in GBP.
    pub fn gbp(amount: Decimal) -> Self {
        Self {
            amount,
            currency: Currency::GBP,
            gbp: amount,
        }
    }

    /// Create a new CurrencyAmount with a foreign currency.
    /// The GBP amount must be provided (from FX conversion).
    pub fn foreign(amount: Decimal, currency: Currency, gbp: Decimal) -> Self {
        debug_assert!(
            currency != Currency::GBP,
            "Use CurrencyAmount::gbp() for GBP"
        );
        Self {
            amount,
            currency,
            gbp,
        }
    }

    /// Check if this amount is in GBP.
    pub fn is_gbp(&self) -> bool {
        self.currency == Currency::GBP
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
        Self::gbp(Decimal::ZERO)
    }
}

// Custom serialization - always serialize as object for consistency
impl Serialize for CurrencyAmount {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;

        let mut state = serializer.serialize_struct("CurrencyAmount", 3)?;
        state.serialize_field("amount", &self.amount)?;
        state.serialize_field("currency", self.currency.code())?;
        state.serialize_field("gbp", &self.gbp)?;
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
                Ok(CurrencyAmount::gbp(amount))
            }

            fn visit_i64<E>(self, v: i64) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                Ok(CurrencyAmount::gbp(Decimal::from(v)))
            }

            fn visit_u64<E>(self, v: u64) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                Ok(CurrencyAmount::gbp(Decimal::from(v)))
            }

            fn visit_f64<E>(self, v: f64) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                use std::str::FromStr;
                let amount = Decimal::from_str(&v.to_string()).map_err(E::custom)?;
                Ok(CurrencyAmount::gbp(amount))
            }

            fn visit_map<M>(self, mut map: M) -> Result<Self::Value, M::Error>
            where
                M: MapAccess<'de>,
            {
                let mut amount: Option<Decimal> = None;
                let mut currency: Option<String> = None;
                let mut gbp: Option<Decimal> = None;

                while let Some(key) = map.next_key::<String>()? {
                    match key.as_str() {
                        "amount" => amount = Some(map.next_value()?),
                        "currency" => currency = Some(map.next_value()?),
                        "gbp" => gbp = Some(map.next_value()?),
                        _ => {
                            let _: serde::de::IgnoredAny = map.next_value()?;
                        }
                    }
                }

                let amount = amount.ok_or_else(|| serde::de::Error::missing_field("amount"))?;
                let currency_code =
                    currency.ok_or_else(|| serde::de::Error::missing_field("currency"))?;
                let gbp = gbp.ok_or_else(|| serde::de::Error::missing_field("gbp"))?;

                let currency = Currency::from_code(&currency_code).ok_or_else(|| {
                    serde::de::Error::custom(format!("invalid currency code: '{currency_code}'"))
                })?;

                if currency == Currency::GBP {
                    Ok(CurrencyAmount::gbp(amount))
                } else {
                    Ok(CurrencyAmount::foreign(amount, currency, gbp))
                }
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
        use schemars::schema::{InstanceType, Schema, SchemaObject, SingleOrVec};

        // CurrencyAmount can be either a plain decimal (for GBP) or an object
        let decimal_schema = generator.subschema_for::<Decimal>();

        let obj_schema = SchemaObject {
            instance_type: Some(SingleOrVec::Single(Box::new(InstanceType::Object))),
            ..Default::default()
        };

        Schema::Object(SchemaObject {
            subschemas: Some(Box::new(schemars::schema::SubschemaValidation {
                any_of: Some(vec![decimal_schema, Schema::Object(obj_schema)]),
                ..Default::default()
            })),
            metadata: Some(Box::new(schemars::schema::Metadata {
                description: Some(
                    "A monetary amount: plain number for GBP, or object with currency".to_owned(),
                ),
                ..Default::default()
            })),
            ..Default::default()
        })
    }
}
