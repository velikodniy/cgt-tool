//! Input validation for CGT transactions.
//!
//! This module provides pre-calculation validation to catch invalid inputs
//! before processing, providing clear error messages.

use crate::models::{Operation, Transaction};
use chrono::NaiveDate;
use rust_decimal::Decimal;
use std::collections::HashMap;
use std::fmt;

/// Severity level of a validation issue.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Severity {
    /// Critical issue that prevents calculation.
    Error,
    /// Non-critical issue that doesn't prevent calculation.
    Warning,
}

impl fmt::Display for Severity {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Severity::Error => write!(f, "Error"),
            Severity::Warning => write!(f, "Warning"),
        }
    }
}

/// A single validation issue (error or warning) found during input validation.
#[derive(Debug, Clone)]
pub struct ValidationIssue {
    /// Whether this issue is an error or a warning.
    pub severity: Severity,
    /// Line number in original input (if known).
    pub line: Option<usize>,
    /// Date of the problematic transaction.
    pub date: NaiveDate,
    /// Ticker symbol involved.
    pub ticker: String,
    /// Description of the issue.
    pub message: String,
}

impl fmt::Display for ValidationIssue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(line) = self.line {
            write!(
                f,
                "{} (line {}): {} on {} - {}",
                self.severity, line, self.ticker, self.date, self.message
            )
        } else {
            write!(
                f,
                "{}: {} on {} - {}",
                self.severity, self.ticker, self.date, self.message
            )
        }
    }
}

/// Result of validating a transaction list.
#[derive(Debug, Clone, Default)]
pub struct ValidationResult {
    /// Critical errors that prevent calculation.
    pub errors: Vec<ValidationIssue>,
    /// Warnings that don't prevent calculation but may indicate issues.
    pub warnings: Vec<ValidationIssue>,
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
                // Check zero quantity
                if *amount == Decimal::ZERO {
                    result.errors.push(ValidationIssue {
                        severity: Severity::Error,
                        line,
                        date: tx.date,
                        ticker: tx.ticker.clone(),
                        message: "BUY with zero quantity".to_string(),
                    });
                }

                // Check negative quantity
                if *amount < Decimal::ZERO {
                    result.errors.push(ValidationIssue {
                        severity: Severity::Error,
                        line,
                        date: tx.date,
                        ticker: tx.ticker.clone(),
                        message: format!("BUY with negative quantity: {}", amount),
                    });
                }

                // Check negative price (GBP value)
                if price.amount < Decimal::ZERO {
                    result.errors.push(ValidationIssue {
                        severity: Severity::Error,
                        line,
                        date: tx.date,
                        ticker: tx.ticker.clone(),
                        message: format!("BUY with negative price: {}", price.amount),
                    });
                }

                // Check negative fees (GBP value)
                if fees.amount < Decimal::ZERO {
                    result.errors.push(ValidationIssue {
                        severity: Severity::Error,
                        line,
                        date: tx.date,
                        ticker: tx.ticker.clone(),
                        message: format!("BUY with negative fees: {}", fees.amount),
                    });
                }

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
                // Check zero quantity
                if *amount == Decimal::ZERO {
                    result.errors.push(ValidationIssue {
                        severity: Severity::Error,
                        line,
                        date: tx.date,
                        ticker: tx.ticker.clone(),
                        message: "SELL with zero quantity".to_string(),
                    });
                }

                // Check negative quantity
                if *amount < Decimal::ZERO {
                    result.errors.push(ValidationIssue {
                        severity: Severity::Error,
                        line,
                        date: tx.date,
                        ticker: tx.ticker.clone(),
                        message: format!("SELL with negative quantity: {}", amount),
                    });
                }

                // Check negative price (GBP value)
                if price.amount < Decimal::ZERO {
                    result.errors.push(ValidationIssue {
                        severity: Severity::Error,
                        line,
                        date: tx.date,
                        ticker: tx.ticker.clone(),
                        message: format!("SELL with negative price: {}", price.amount),
                    });
                }

                // Check negative fees (GBP value)
                if fees.amount < Decimal::ZERO {
                    result.errors.push(ValidationIssue {
                        severity: Severity::Error,
                        line,
                        date: tx.date,
                        ticker: tx.ticker.clone(),
                        message: format!("SELL with negative fees: {}", fees.amount),
                    });
                }

                // Check for sell before buy (warning)
                if let Some(&first_buy_date) = first_buy.get(tx.ticker.as_str()) {
                    if tx.date < first_buy_date {
                        result.warnings.push(ValidationIssue {
                            severity: Severity::Warning,
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
                    result.warnings.push(ValidationIssue {
                        severity: Severity::Warning,
                        line,
                        date: tx.date,
                        ticker: tx.ticker.clone(),
                        message: "SELL with no prior BUY for this ticker".to_string(),
                    });
                }
            }

            Operation::Split { ratio } => {
                // Check zero ratio
                if *ratio == Decimal::ZERO {
                    result.errors.push(ValidationIssue {
                        severity: Severity::Error,
                        line,
                        date: tx.date,
                        ticker: tx.ticker.clone(),
                        message: "SPLIT with zero ratio".to_string(),
                    });
                }

                // Check negative ratio
                if *ratio < Decimal::ZERO {
                    result.errors.push(ValidationIssue {
                        severity: Severity::Error,
                        line,
                        date: tx.date,
                        ticker: tx.ticker.clone(),
                        message: format!("SPLIT with negative ratio: {}", ratio),
                    });
                }
            }

            Operation::Unsplit { ratio } => {
                // Check zero ratio
                if *ratio == Decimal::ZERO {
                    result.errors.push(ValidationIssue {
                        severity: Severity::Error,
                        line,
                        date: tx.date,
                        ticker: tx.ticker.clone(),
                        message: "UNSPLIT with zero ratio".to_string(),
                    });
                }

                // Check negative ratio
                if *ratio < Decimal::ZERO {
                    result.errors.push(ValidationIssue {
                        severity: Severity::Error,
                        line,
                        date: tx.date,
                        ticker: tx.ticker.clone(),
                        message: format!("UNSPLIT with negative ratio: {}", ratio),
                    });
                }
            }

            Operation::Dividend {
                amount,
                total_value,
                ..
            } => {
                // Check zero quantity
                if *amount == Decimal::ZERO {
                    result.errors.push(ValidationIssue {
                        severity: Severity::Error,
                        line,
                        date: tx.date,
                        ticker: tx.ticker.clone(),
                        message: "DIVIDEND with zero quantity".to_string(),
                    });
                }

                // Check negative quantity
                if *amount < Decimal::ZERO {
                    result.errors.push(ValidationIssue {
                        severity: Severity::Error,
                        line,
                        date: tx.date,
                        ticker: tx.ticker.clone(),
                        message: format!("DIVIDEND with negative quantity: {}", amount),
                    });
                }

                // Check negative total value (GBP value)
                if total_value.amount < Decimal::ZERO {
                    result.errors.push(ValidationIssue {
                        severity: Severity::Error,
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

            Operation::CapReturn {
                amount,
                total_value,
                fees,
            } => {
                // Check zero quantity
                if *amount == Decimal::ZERO {
                    result.errors.push(ValidationIssue {
                        severity: Severity::Error,
                        line,
                        date: tx.date,
                        ticker: tx.ticker.clone(),
                        message: "CAPRETURN with zero quantity".to_string(),
                    });
                }

                // Check negative quantity
                if *amount < Decimal::ZERO {
                    result.errors.push(ValidationIssue {
                        severity: Severity::Error,
                        line,
                        date: tx.date,
                        ticker: tx.ticker.clone(),
                        message: format!("CAPRETURN with negative quantity: {}", amount),
                    });
                }

                // Check negative total value (GBP value)
                if total_value.amount < Decimal::ZERO {
                    result.errors.push(ValidationIssue {
                        severity: Severity::Error,
                        line,
                        date: tx.date,
                        ticker: tx.ticker.clone(),
                        message: format!(
                            "CAPRETURN with negative total value: {}",
                            total_value.amount
                        ),
                    });
                }

                // Check negative fees (GBP value)
                if fees.amount < Decimal::ZERO {
                    result.errors.push(ValidationIssue {
                        severity: Severity::Error,
                        line,
                        date: tx.date,
                        ticker: tx.ticker.clone(),
                        message: format!("CAPRETURN with negative fees: {}", fees.amount),
                    });
                }
            }
        }
    }

    result
}
