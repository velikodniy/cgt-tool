//! Input validation for CGT transactions.
//!
//! This module provides pre-calculation validation to catch invalid inputs
//! before processing, providing clear error messages.

use crate::model::{Operation, Transaction, is_valid_ticker};
use chrono::NaiveDate;
use rust_decimal::Decimal;
use serde::Serialize;
use serde::ser::SerializeStruct;
use std::collections::{HashMap, HashSet};
use std::fmt;

/// Result of validating a transaction list.
///
/// Serializes with an additional `is_valid` field derived from [`Self::is_valid`].
#[derive(Debug, Clone, Default)]
pub struct ValidationResult {
    /// Critical errors that prevent calculation.
    pub errors: Vec<ValidationError>,
    /// Warnings that don't prevent calculation but may indicate issues.
    pub warnings: Vec<ValidationWarning>,
}

impl Serialize for ValidationResult {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut state = serializer.serialize_struct("ValidationResult", 3)?;
        state.serialize_field("is_valid", &self.is_valid())?;
        state.serialize_field("errors", &self.errors)?;
        state.serialize_field("warnings", &self.warnings)?;
        state.end()
    }
}

impl ValidationResult {
    /// Returns true if there are no errors (warnings are allowed).
    pub fn is_valid(&self) -> bool {
        self.errors.is_empty()
    }

    /// Returns true if there are no errors or warnings.
    pub fn is_clean(&self) -> bool {
        self.errors.is_empty() && self.warnings.is_empty()
    }
}

/// A validation error that prevents calculation.
#[derive(Debug, Clone, Serialize)]
pub struct ValidationError {
    /// Line number in original input (if known).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub line: Option<usize>,
    /// Date of the problematic transaction.
    pub date: NaiveDate,
    /// Ticker symbol involved.
    pub ticker: String,
    /// Description of the error.
    pub message: String,
}

impl fmt::Display for ValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(line) = self.line {
            write!(
                f,
                "Error (line {}): {} on {} - {}",
                line, self.ticker, self.date, self.message
            )
        } else {
            write!(
                f,
                "Error: {} on {} - {}",
                self.ticker, self.date, self.message
            )
        }
    }
}

/// A validation warning that doesn't prevent calculation.
#[derive(Debug, Clone, Serialize)]
pub struct ValidationWarning {
    /// Line number in original input (if known).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub line: Option<usize>,
    /// Date of the problematic transaction.
    pub date: NaiveDate,
    /// Ticker symbol involved.
    pub ticker: String,
    /// Description of the warning.
    pub message: String,
}

impl fmt::Display for ValidationWarning {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(line) = self.line {
            write!(
                f,
                "Warning (line {}): {} on {} - {}",
                line, self.ticker, self.date, self.message
            )
        } else {
            write!(
                f,
                "Warning: {} on {} - {}",
                self.ticker, self.date, self.message
            )
        }
    }
}

fn nonnegative(errors: &mut Vec<String>, action: &str, field: &str, value: Decimal) {
    if value < Decimal::ZERO {
        errors.push(format!("{action} with negative {field}: {value}"));
    }
}

fn positive(errors: &mut Vec<String>, action: &str, field: &str, value: Decimal) {
    if value.is_zero() {
        errors.push(format!("{action} with zero {field}"));
    }
    nonnegative(errors, action, field, value);
}

/// Validate one transaction without applying rules that depend on other records.
pub(crate) fn transaction_errors(transaction: &Transaction) -> Vec<ValidationError> {
    let mut errors = Vec::new();
    if !is_valid_ticker(&transaction.ticker) {
        errors.push("Invalid ticker symbol".to_string());
    }
    match &transaction.operation {
        Operation::Buy {
            amount,
            price,
            fees,
        }
        | Operation::Sell {
            amount,
            price,
            fees,
        } => {
            let action = if matches!(transaction.operation, Operation::Buy { .. }) {
                "BUY"
            } else {
                "SELL"
            };
            positive(&mut errors, action, "quantity", *amount);
            nonnegative(&mut errors, action, "price", price.amount);
            nonnegative(&mut errors, action, "fees", fees.amount);
        }
        Operation::Dividend {
            total_value,
            tax_paid,
        } => {
            nonnegative(&mut errors, "DIVIDEND", "total value", total_value.amount);
            nonnegative(&mut errors, "DIVIDEND", "tax", tax_paid.amount);
        }
        Operation::Accumulation {
            amount,
            total_value,
            tax_paid,
        } => {
            // Zero quantity remains a no-op. Negative fields are invalid.
            nonnegative(&mut errors, "ACCUMULATION", "quantity", *amount);
            nonnegative(
                &mut errors,
                "ACCUMULATION",
                "total value",
                total_value.amount,
            );
            nonnegative(&mut errors, "ACCUMULATION", "tax", tax_paid.amount);
        }
        Operation::CapReturn {
            amount,
            total_value,
            fees,
        } => {
            positive(&mut errors, "CAPRETURN", "quantity", *amount);
            nonnegative(&mut errors, "CAPRETURN", "total value", total_value.amount);
            nonnegative(&mut errors, "CAPRETURN", "fees", fees.amount);
        }
        Operation::Split { ratio } => positive(&mut errors, "SPLIT", "ratio", *ratio),
        Operation::Unsplit { ratio } => {
            positive(&mut errors, "UNSPLIT", "ratio", *ratio);
            if *ratio > Decimal::ZERO && !ratio.fract().is_zero() {
                errors.push(format!("UNSPLIT with non-integer ratio: {ratio}"));
            }
        }
    }
    errors
        .into_iter()
        .map(|message| ValidationError {
            line: transaction.line,
            date: transaction.date,
            ticker: transaction.ticker.clone(),
            message,
        })
        .collect()
}

/// Validate transaction fields, cross-record conflicts, and sale history.
/// Errors prevent calculation. Warnings retain the source location.
pub fn validate(transactions: &[Transaction]) -> ValidationResult {
    let mut result = ValidationResult::default();
    let mut first_buy: HashMap<&str, NaiveDate> = HashMap::new();
    for transaction in transactions {
        if matches!(transaction.operation, Operation::Buy { .. }) {
            first_buy
                .entry(transaction.ticker.as_str())
                .and_modify(|date| *date = (*date).min(transaction.date))
                .or_insert(transaction.date);
        }
    }
    for transaction in transactions {
        result.errors.extend(transaction_errors(transaction));
        if matches!(transaction.operation, Operation::Sell { .. }) {
            let message = match first_buy.get(transaction.ticker.as_str()) {
                Some(first_buy_date) if transaction.date < *first_buy_date => Some(format!(
                    "SELL before first BUY (first buy: {first_buy_date})"
                )),
                None => Some("SELL with no prior BUY for this ticker".to_string()),
                Some(_) => None,
            };
            if let Some(message) = message {
                result.warnings.push(ValidationWarning {
                    line: transaction.line,
                    date: transaction.date,
                    ticker: transaction.ticker.clone(),
                    message,
                });
            }
        }
    }
    check_reorganisation_collisions(&mut result, transactions);
    result
}

/// Reject a SPLIT/UNSPLIT on the same date as a BUY or SELL of the same ticker.
///
/// A split is a reorganisation of share capital (TCGA92/S127): not a disposal
/// or acquisition, so it has no defined intra-day order against a same-date
/// BUY or SELL in pre- or post-split shares, which would yield a materially
/// different holding. That collision is rejected as ambiguous. ACCUMULATION,
/// CAPRETURN, and DIVIDEND only adjust cost or income (not the share count),
/// so they commute with a split and are not flagged.
fn check_reorganisation_collisions(result: &mut ValidationResult, transactions: &[Transaction]) {
    let mut reorg_dates: HashSet<(NaiveDate, &str)> = HashSet::new();
    for tx in transactions {
        if matches!(
            tx.operation,
            Operation::Split { .. } | Operation::Unsplit { .. }
        ) {
            reorg_dates.insert((tx.date, tx.ticker.as_str()));
        }
    }

    for tx in transactions {
        if matches!(tx.operation, Operation::Buy { .. } | Operation::Sell { .. })
            && reorg_dates.contains(&(tx.date, tx.ticker.as_str()))
        {
            result.errors.push(ValidationError {
                line: tx.line,
                date: tx.date,
                ticker: tx.ticker.clone(),
                message: "a BUY or SELL on the same date as a SPLIT/UNSPLIT of the same \
                          ticker is ambiguous (no defined order between the reorganisation \
                          and the change in holding); date it before or after the split"
                    .to_string(),
            });
        }
    }
}
