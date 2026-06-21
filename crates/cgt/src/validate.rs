//! Input validation for CGT transactions.
//!
//! This module provides pre-calculation validation to catch invalid inputs
//! before processing, providing clear error messages.

use crate::model::{Operation, Transaction};
use crate::money::CurrencyAmount;
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

/// Fields for a trade-like operation (Buy, Sell, CapReturn) to be validated.
struct TradeFields<'a> {
    action: &'a str,
    amount: Decimal,
    price: &'a CurrencyAmount,
    price_label: &'a str,
    fees: &'a CurrencyAmount,
}

/// Check quantity, price, and fees fields common to Buy, Sell, and CapReturn.
fn check_trade_fields(
    result: &mut ValidationResult,
    line: Option<usize>,
    date: NaiveDate,
    ticker: &str,
    fields: &TradeFields<'_>,
) {
    if fields.amount == Decimal::ZERO {
        result.errors.push(ValidationError {
            line,
            date,
            ticker: ticker.to_string(),
            message: format!("{} with zero quantity", fields.action),
        });
    }

    if fields.amount < Decimal::ZERO {
        result.errors.push(ValidationError {
            line,
            date,
            ticker: ticker.to_string(),
            message: format!(
                "{} with negative quantity: {}",
                fields.action, fields.amount
            ),
        });
    }

    if fields.price.amount < Decimal::ZERO {
        result.errors.push(ValidationError {
            line,
            date,
            ticker: ticker.to_string(),
            message: format!(
                "{} with negative {}: {}",
                fields.action, fields.price_label, fields.price.amount
            ),
        });
    }

    if fields.fees.amount < Decimal::ZERO {
        result.errors.push(ValidationError {
            line,
            date,
            ticker: ticker.to_string(),
            message: format!(
                "{} with negative fees: {}",
                fields.action, fields.fees.amount
            ),
        });
    }
}

/// Validate a list of transactions before calculation.
///
/// Checks for:
/// - Zero-quantity transactions
/// - Negative prices and expenses
/// - Zero/negative split ratios
/// - Sells before any buys (warning)
///
/// # Arguments
/// * `transactions` - The transactions to validate
///
/// # Returns
/// A `ValidationResult` containing any errors and warnings found.
pub fn validate(transactions: &[Transaction]) -> ValidationResult {
    let mut result = ValidationResult::default();

    // Track first buy date per ticker for "sell before buy" warning
    let mut first_buy: HashMap<&str, NaiveDate> = HashMap::new();

    for (i, tx) in transactions.iter().enumerate() {
        let line = Some(i + 1);

        match &tx.operation {
            Operation::Buy {
                amount,
                price,
                fees,
            } => {
                check_trade_fields(
                    &mut result,
                    line,
                    tx.date,
                    &tx.ticker,
                    &TradeFields {
                        action: "BUY",
                        amount: *amount,
                        price,
                        price_label: "price",
                        fees,
                    },
                );

                // Track first buy date
                first_buy
                    .entry(&tx.ticker)
                    .and_modify(|d| {
                        if tx.date < *d {
                            *d = tx.date;
                        }
                    })
                    .or_insert(tx.date);
            }

            Operation::Sell {
                amount,
                price,
                fees,
            } => {
                check_trade_fields(
                    &mut result,
                    line,
                    tx.date,
                    &tx.ticker,
                    &TradeFields {
                        action: "SELL",
                        amount: *amount,
                        price,
                        price_label: "price",
                        fees,
                    },
                );

                // Check for sell before buy (warning)
                if let Some(&first_buy_date) = first_buy.get(tx.ticker.as_str()) {
                    if tx.date < first_buy_date {
                        result.warnings.push(ValidationWarning {
                            line,
                            date: tx.date,
                            ticker: tx.ticker.clone(),
                            message: format!(
                                "SELL before first BUY (first buy: {})",
                                first_buy_date
                            ),
                        });
                    }
                } else {
                    result.warnings.push(ValidationWarning {
                        line,
                        date: tx.date,
                        ticker: tx.ticker.clone(),
                        message: "SELL with no prior BUY for this ticker".to_string(),
                    });
                }
            }

            Operation::Split { ratio } => {
                if *ratio == Decimal::ZERO {
                    result.errors.push(ValidationError {
                        line,
                        date: tx.date,
                        ticker: tx.ticker.clone(),
                        message: "SPLIT with zero ratio".to_string(),
                    });
                }

                if *ratio < Decimal::ZERO {
                    result.errors.push(ValidationError {
                        line,
                        date: tx.date,
                        ticker: tx.ticker.clone(),
                        message: format!("SPLIT with negative ratio: {}", ratio),
                    });
                }
            }

            Operation::Unsplit { ratio } => {
                if *ratio == Decimal::ZERO {
                    result.errors.push(ValidationError {
                        line,
                        date: tx.date,
                        ticker: tx.ticker.clone(),
                        message: "UNSPLIT with zero ratio".to_string(),
                    });
                }

                if *ratio < Decimal::ZERO {
                    result.errors.push(ValidationError {
                        line,
                        date: tx.date,
                        ticker: tx.ticker.clone(),
                        message: format!("UNSPLIT with negative ratio: {}", ratio),
                    });
                }
            }

            Operation::Dividend { total_value, .. } => {
                if total_value.amount < Decimal::ZERO {
                    result.errors.push(ValidationError {
                        line,
                        date: tx.date,
                        ticker: tx.ticker.clone(),
                        message: format!(
                            "DIVIDEND with negative total value: {}",
                            total_value.amount
                        ),
                    });
                }
            }

            Operation::Accumulation {
                amount,
                total_value,
                ..
            } => {
                // A zero-quantity accumulation is a no-op: no units accrue, so
                // it neither matches nor adjusts a pool. Only a negative
                // quantity is invalid.
                if *amount < Decimal::ZERO {
                    result.errors.push(ValidationError {
                        line,
                        date: tx.date,
                        ticker: tx.ticker.clone(),
                        message: format!("ACCUMULATION with negative quantity: {}", amount),
                    });
                }

                if total_value.amount < Decimal::ZERO {
                    result.errors.push(ValidationError {
                        line,
                        date: tx.date,
                        ticker: tx.ticker.clone(),
                        message: format!(
                            "ACCUMULATION with negative total value: {}",
                            total_value.amount
                        ),
                    });
                }
            }

            Operation::CapReturn {
                amount,
                total_value,
                fees,
            } => {
                check_trade_fields(
                    &mut result,
                    line,
                    tx.date,
                    &tx.ticker,
                    &TradeFields {
                        action: "CAPRETURN",
                        amount: *amount,
                        price: total_value,
                        price_label: "total value",
                        fees,
                    },
                );
            }
        }
    }

    check_reorganisation_collisions(&mut result, transactions);

    result
}

/// Reject a SPLIT/UNSPLIT on the same date as a holding-changing event (BUY,
/// SELL, or ACCUMULATION) of the same ticker.
///
/// A split is a reorganisation of share capital (TCGA92/S127): not a disposal
/// or acquisition, so it does not enter the same-day matching rules and HMRC
/// gives no intra-day order relative to a same-date event that changes the
/// holding. A BUY/SELL in pre- or post-split shares, or an ACCUMULATION whose
/// units accrue before or after the split, yields a materially different
/// holding, so the input is ambiguous and the user must date the event before
/// or after the split. CAPRETURN and DIVIDEND change no share count, so their
/// order relative to the split is immaterial and they are not flagged.
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

    for (i, tx) in transactions.iter().enumerate() {
        if matches!(
            tx.operation,
            Operation::Buy { .. } | Operation::Sell { .. } | Operation::Accumulation { .. }
        ) && reorg_dates.contains(&(tx.date, tx.ticker.as_str()))
        {
            result.errors.push(ValidationError {
                line: Some(i + 1),
                date: tx.date,
                ticker: tx.ticker.clone(),
                message: "a BUY, SELL, or ACCUMULATION on the same date as a SPLIT/UNSPLIT of \
                          the same ticker is ambiguous (no defined order between the \
                          reorganisation and the change in holding); date it before or after \
                          the split"
                    .to_string(),
            });
        }
    }
}
