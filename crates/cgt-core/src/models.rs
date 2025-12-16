use chrono::{Datelike, NaiveDate};
use rust_decimal::Decimal;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::error::CgtError;

// Re-export CurrencyAmount from cgt-money
pub use cgt_money::CurrencyAmount;

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
        fees: CurrencyAmount,
    },
    Sell {
        amount: Decimal,
        price: CurrencyAmount,
        fees: CurrencyAmount,
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
        fees: CurrencyAmount,
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
