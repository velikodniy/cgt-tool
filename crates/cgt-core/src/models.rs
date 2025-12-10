use chrono::{Datelike, NaiveDate};
use iso_currency::Currency;
use rust_decimal::Decimal;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::error::CgtError;

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

/// A validated UK tax year identifier (April 6 to April 5).
///
/// Stores the start year internally and serializes to "YYYY/YY" format (e.g., "2023/24").
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct TaxPeriod(u16);

impl TaxPeriod {
    /// Create a new TaxPeriod from a start year.
    ///
    /// # Errors
    /// Returns `CgtError::InvalidTaxYear` if the year is outside the range 1900-2100.
    pub fn new(start_year: u16) -> Result<Self, CgtError> {
        if !(1900..=2100).contains(&start_year) {
            return Err(CgtError::InvalidTaxYear(start_year));
        }
        Ok(Self(start_year))
    }

    /// Derive the tax year from a date.
    ///
    /// UK tax year starts April 6, so:
    /// - 2024-03-15 → "2023/24" (before April 6)
    /// - 2024-04-10 → "2024/25" (on or after April 6)
    pub fn from_date(date: NaiveDate) -> Self {
        let year = date.year() as u16;
        let month = date.month();
        let day = date.day();
        if month < 4 || (month == 4 && day < 6) {
            Self(year - 1)
        } else {
            Self(year)
        }
    }

    /// Get the start year of this tax period.
    pub fn start_year(&self) -> u16 {
        self.0
    }

    /// Get the end year of this tax period (always start_year + 1).
    pub fn end_year(&self) -> u16 {
        self.0 + 1
    }

    /// Get the start date of this tax year (April 6 of start year).
    pub fn start_date(&self) -> Option<NaiveDate> {
        NaiveDate::from_ymd_opt(self.0 as i32, 4, 6)
    }

    /// Get the end date of this tax year (April 5 of end year).
    pub fn end_date(&self) -> Option<NaiveDate> {
        NaiveDate::from_ymd_opt(self.end_year() as i32, 4, 5)
    }
}

impl Serialize for TaxPeriod {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let end_short = (self.0 + 1) % 100;
        serializer.serialize_str(&format!("{}/{:02}", self.0, end_short))
    }
}

impl<'de> Deserialize<'de> for TaxPeriod {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        let parts: Vec<&str> = s.split('/').collect();
        if parts.len() != 2 {
            return Err(serde::de::Error::custom(format!(
                "invalid tax period format: expected 'YYYY/YY', got '{s}'"
            )));
        }
        let start: u16 = parts[0]
            .parse()
            .map_err(|_| serde::de::Error::custom(format!("invalid start year: '{}'", parts[0])))?;
        let end_short: u16 = parts[1]
            .parse()
            .map_err(|_| serde::de::Error::custom(format!("invalid end year: '{}'", parts[1])))?;

        let expected_end = (start + 1) % 100;
        if end_short != expected_end {
            return Err(serde::de::Error::custom(format!(
                "tax years must be consecutive: '{s}' should end with '{expected_end:02}', not '{end_short:02}'"
            )));
        }

        TaxPeriod::new(start).map_err(serde::de::Error::custom)
    }
}

impl JsonSchema for TaxPeriod {
    fn schema_name() -> String {
        "TaxPeriod".to_owned()
    }

    fn json_schema(generator: &mut schemars::r#gen::SchemaGenerator) -> schemars::schema::Schema {
        let mut schema = generator.subschema_for::<String>().into_object();
        schema.metadata().description =
            Some("UK tax year in 'YYYY/YY' format (e.g., '2023/24')".to_owned());
        schema.string().pattern = Some(r"^\d{4}/\d{2}$".to_owned());
        schema.into()
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct Transaction {
    pub date: NaiveDate,
    pub ticker: String,
    #[serde(flatten)]
    pub operation: Operation,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "action", rename_all = "SCREAMING_SNAKE_CASE")]
pub enum Operation {
    Buy {
        amount: Decimal,
        price: CurrencyAmount,
        expenses: CurrencyAmount,
    },
    Sell {
        amount: Decimal,
        price: CurrencyAmount,
        expenses: CurrencyAmount,
    },
    Dividend {
        amount: Decimal,
        total_value: CurrencyAmount,
        tax_paid: CurrencyAmount,
    },
    #[serde(rename = "CAPRETURN")]
    CapReturn {
        amount: Decimal,
        total_value: CurrencyAmount,
        expenses: CurrencyAmount,
    },
    Split {
        ratio: Decimal,
    },
    Unsplit {
        ratio: Decimal,
    },
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, JsonSchema)]
pub struct Section104Holding {
    pub ticker: String,
    pub quantity: Decimal,
    pub total_cost: Decimal,
}

/// Enumeration of HMRC share matching rules.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, JsonSchema)]
pub enum MatchRule {
    SameDay,
    BedAndBreakfast,
    Section104,
}

/// How a disposal (or portion) was matched to an acquisition.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, JsonSchema)]
pub struct Match {
    pub rule: MatchRule,
    pub quantity: Decimal,
    pub allowable_cost: Decimal,
    pub gain_or_loss: Decimal,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub acquisition_date: Option<NaiveDate>,
}

/// A sale event that triggers CGT calculation.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, JsonSchema)]
pub struct Disposal {
    pub date: NaiveDate,
    pub ticker: String,
    pub quantity: Decimal,
    pub proceeds: Decimal,
    pub matches: Vec<Match>,
}

/// Summary of CGT activity within a single UK tax year.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, JsonSchema)]
pub struct TaxYearSummary {
    pub period: TaxPeriod,
    pub disposals: Vec<Disposal>,
    pub total_gain: Decimal,
    pub total_loss: Decimal,
    pub net_gain: Decimal,
}

/// The complete CGT calculation output.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, JsonSchema)]
pub struct TaxReport {
    pub tax_years: Vec<TaxYearSummary>,
    pub holdings: Vec<Section104Holding>,
}
