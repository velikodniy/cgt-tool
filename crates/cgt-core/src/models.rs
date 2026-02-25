use cgt_money::FxCache;
use chrono::{Datelike, NaiveDate};
use rust_decimal::Decimal;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::error::CgtError;

// Re-export Currency and CurrencyAmount from cgt-money
pub use cgt_money::{Currency, CurrencyAmount};

/// Serialize a Decimal to at most 2 decimal places for monetary amounts.
mod decimal_money {
    use rust_decimal::Decimal;
    use serde::{self, Serializer};

    pub fn serialize<S>(value: &Decimal, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        // Round to 2 decimal places for display
        let rounded = value.round_dp(2);
        serializer.serialize_str(&rounded.to_string())
    }
}

/// Deserialize Operation with case-insensitive action names.
fn deserialize_operation_value(
    mut value: serde_json::Value,
) -> Result<Operation<CurrencyAmount>, String> {
    normalize_operation_action(&mut value)?;
    let operation: Operation<CurrencyAmount> =
        serde_json::from_value(value).map_err(|e| format!("invalid transaction: {e}"))?;
    validate_operation(&operation)?;
    Ok(operation)
}

fn normalize_operation_action(value: &mut serde_json::Value) -> Result<(), String> {
    let Some(action) = value.get_mut("action") else {
        return Err("invalid transaction: missing 'action' field".to_string());
    };
    let Some(action_str) = action.as_str() else {
        return Err("invalid transaction: 'action' must be a string".to_string());
    };
    let normalized = match action_str.to_uppercase().as_str() {
        "CAP_RETURN" => "CAPRETURN".to_string(),
        other => other.to_string(),
    };
    *action = serde_json::Value::String(normalized);
    Ok(())
}

fn validate_operation(operation: &Operation<CurrencyAmount>) -> Result<(), String> {
    match operation {
        Operation::Buy { amount, .. } => validate_positive(*amount, "amount", "BUY"),
        Operation::Sell { amount, .. } => validate_positive(*amount, "amount", "SELL"),
        Operation::Dividend { amount, .. } => validate_positive(*amount, "amount", "DIVIDEND"),
        Operation::CapReturn { amount, .. } => validate_positive(*amount, "amount", "CAPRETURN"),
        Operation::Split { ratio } => validate_positive_ratio(*ratio, "SPLIT"),
        Operation::Unsplit { ratio } => validate_positive_ratio(*ratio, "UNSPLIT"),
    }
}

/// Validate that an amount is positive.
fn validate_positive(amount: Decimal, field: &str, action: &str) -> Result<(), String> {
    if amount <= Decimal::ZERO {
        return Err(format!(
            "{action} action: '{field}' must be positive (got {amount}). \
             Negative amounts are not supported."
        ));
    }
    Ok(())
}

/// Validate that a ratio is positive.
fn validate_positive_ratio(ratio: Decimal, action: &str) -> Result<(), String> {
    if ratio <= Decimal::ZERO {
        return Err(format!(
            "{action} action: 'ratio' must be positive (got {ratio})"
        ));
    }
    Ok(())
}

/// A validated UK tax year identifier (April 6 to April 5).
///
/// Stores the start year internally and serializes to "YYYY/YY" format (e.g., "2023/24").
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct TaxPeriod(u16);

const MIN_TAX_YEAR: u16 = 1900;
const MAX_TAX_YEAR: u16 = 2100;

impl TaxPeriod {
    /// Create a new TaxPeriod from a start year.
    ///
    /// # Errors
    /// Returns `CgtError::InvalidTaxYear` if the year is outside the valid range.
    pub fn new(start_year: u16) -> Result<Self, CgtError> {
        if !(MIN_TAX_YEAR..=MAX_TAX_YEAR).contains(&start_year) {
            return Err(CgtError::InvalidTaxYear(start_year));
        }
        Ok(Self(start_year))
    }

    /// Derive the tax year from a date.
    ///
    /// UK tax year starts April 6, so:
    /// - 2024-03-15 → "2023/24" (before April 6)
    /// - 2024-04-10 → "2024/25" (on or after April 6)
    ///
    /// # Errors
    /// Returns `CgtError::InvalidTaxYear` if the derived year is outside the valid range.
    pub fn from_date(date: NaiveDate) -> Result<Self, CgtError> {
        let year = date.year() as u16;
        let month = date.month();
        let day = date.day();
        let start_year = if month < 4 || (month == 4 && day < 6) {
            year - 1
        } else {
            year
        };
        Self::new(start_year)
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

impl std::fmt::Display for TaxPeriod {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let end_short = (self.0 + 1) % 100;
        write!(f, "{}/{:02}", self.0, end_short)
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

/// A transaction with amounts in their original currency.
/// Used for parsing and JSON I/O.
#[derive(Debug, Clone, PartialEq, Serialize, JsonSchema)]
pub struct Transaction {
    pub date: NaiveDate,
    pub ticker: String,
    #[serde(flatten)]
    pub operation: Operation<CurrencyAmount>,
}

impl<'de> Deserialize<'de> for Transaction {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        #[derive(Deserialize)]
        struct RawTransaction {
            date: NaiveDate,
            ticker: String,
            #[serde(flatten)]
            operation: serde_json::Value,
        }

        let raw = RawTransaction::deserialize(deserializer)?;
        let operation: Operation<CurrencyAmount> =
            deserialize_operation_value(raw.operation).map_err(serde::de::Error::custom)?;

        Ok(Transaction {
            date: raw.date,
            ticker: raw.ticker.to_uppercase(),
            operation,
        })
    }
}

/// A transaction with all monetary amounts converted to GBP.
/// Used internally for CGT calculations.
#[derive(Debug, Clone, PartialEq)]
pub struct GbpTransaction {
    pub date: NaiveDate,
    pub ticker: String,
    pub operation: Operation<Decimal>,
}

impl Transaction {
    /// Convert this transaction to a GBP-normalized transaction.
    ///
    /// All monetary amounts are converted to GBP using the FX rate for the transaction date.
    /// If the transaction is already in GBP, no conversion is needed.
    pub fn to_gbp(&self, fx_cache: Option<&FxCache>) -> Result<GbpTransaction, CgtError> {
        let date = self.date;
        let operation = match &self.operation {
            Operation::Buy {
                amount,
                price,
                fees,
            } => {
                let price_gbp = amount_to_gbp(price, date, fx_cache)?;
                let fees_gbp = amount_to_gbp(fees, date, fx_cache)?;
                Operation::Buy {
                    amount: *amount,
                    price: price_gbp,
                    fees: fees_gbp,
                }
            }
            Operation::Sell {
                amount,
                price,
                fees,
            } => {
                let price_gbp = amount_to_gbp(price, date, fx_cache)?;
                let fees_gbp = amount_to_gbp(fees, date, fx_cache)?;
                Operation::Sell {
                    amount: *amount,
                    price: price_gbp,
                    fees: fees_gbp,
                }
            }
            Operation::Dividend {
                amount,
                total_value,
                tax_paid,
            } => {
                let total_value_gbp = amount_to_gbp(total_value, date, fx_cache)?;
                let tax_paid_gbp = amount_to_gbp(tax_paid, date, fx_cache)?;
                Operation::Dividend {
                    amount: *amount,
                    total_value: total_value_gbp,
                    tax_paid: tax_paid_gbp,
                }
            }
            Operation::CapReturn {
                amount,
                total_value,
                fees,
            } => {
                let total_value_gbp = amount_to_gbp(total_value, date, fx_cache)?;
                let fees_gbp = amount_to_gbp(fees, date, fx_cache)?;
                Operation::CapReturn {
                    amount: *amount,
                    total_value: total_value_gbp,
                    fees: fees_gbp,
                }
            }
            Operation::Split { ratio } => Operation::Split { ratio: *ratio },
            Operation::Unsplit { ratio } => Operation::Unsplit { ratio: *ratio },
        };

        Ok(GbpTransaction {
            date: self.date,
            ticker: self.ticker.clone(),
            operation,
        })
    }
}

/// Convert a CurrencyAmount to GBP using the FX cache.
fn amount_to_gbp(
    amount: &CurrencyAmount,
    date: NaiveDate,
    fx_cache: Option<&FxCache>,
) -> Result<Decimal, CgtError> {
    if amount.is_gbp() {
        return Ok(amount.amount);
    }

    let code = amount.code().to_string();
    let cache = fx_cache.ok_or(CgtError::MissingFxRate {
        currency: code.clone(),
        year: date.year(),
        month: date.month(),
    })?;

    amount
        .to_gbp(date, cache)
        .map_err(|_| CgtError::MissingFxRate {
            currency: code,
            year: date.year(),
            month: date.month(),
        })
}

/// Convert a slice of transactions to GBP-normalized transactions.
pub fn transactions_to_gbp(
    transactions: &[Transaction],
    fx_cache: Option<&FxCache>,
) -> Result<Vec<GbpTransaction>, CgtError> {
    transactions.iter().map(|tx| tx.to_gbp(fx_cache)).collect()
}

/// A financial operation, generic over the monetary amount type.
///
/// - `Operation<CurrencyAmount>`: amounts in original currency (for I/O)
/// - `Operation<Decimal>`: amounts in GBP (for calculations)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "action", rename_all = "SCREAMING_SNAKE_CASE")]
pub enum Operation<M: Default> {
    Buy {
        amount: Decimal,
        price: M,
        #[serde(default)]
        fees: M,
    },
    Sell {
        amount: Decimal,
        price: M,
        #[serde(default)]
        fees: M,
    },
    Dividend {
        amount: Decimal,
        total_value: M,
        #[serde(default)]
        tax_paid: M,
    },
    #[serde(rename = "CAPRETURN")]
    CapReturn {
        amount: Decimal,
        total_value: M,
        #[serde(default)]
        fees: M,
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
    #[serde(serialize_with = "decimal_money::serialize")]
    pub total_cost: Decimal,
}

/// Enumeration of HMRC share matching rules (CG51560).
///
/// Applied in priority order: Same Day (S105(1)), Bed & Breakfast (S106A),
/// then Section 104 pool.
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
    #[serde(serialize_with = "decimal_money::serialize")]
    pub allowable_cost: Decimal,
    #[serde(serialize_with = "decimal_money::serialize")]
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
    /// Gross proceeds before sale fees (quantity × unit price). Used for SA108 Box 21.
    #[serde(serialize_with = "decimal_money::serialize")]
    pub gross_proceeds: Decimal,
    /// Net proceeds after sale fees (gross_proceeds - fees). Used for gain calculation.
    #[serde(serialize_with = "decimal_money::serialize")]
    pub proceeds: Decimal,
    pub matches: Vec<Match>,
}

/// Summary of CGT activity within a single UK tax year.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, JsonSchema)]
pub struct TaxYearSummary {
    pub period: TaxPeriod,
    pub disposals: Vec<Disposal>,
    #[serde(default)]
    pub disposal_count: u32,
    #[serde(serialize_with = "decimal_money::serialize")]
    pub total_gain: Decimal,
    #[serde(serialize_with = "decimal_money::serialize")]
    pub total_loss: Decimal,
    #[serde(serialize_with = "decimal_money::serialize")]
    pub net_gain: Decimal,
}

/// The complete CGT calculation output.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, JsonSchema)]
pub struct TaxReport {
    pub tax_years: Vec<TaxYearSummary>,
    pub holdings: Vec<Section104Holding>,
}
