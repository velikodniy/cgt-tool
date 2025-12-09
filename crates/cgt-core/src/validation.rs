//! Input validation for CGT transactions.
//!
//! This module provides pre-calculation validation to catch invalid inputs
//! before processing, providing clear error messages.

use crate::models::{Operation, Transaction};
use chrono::NaiveDate;
use rust_decimal::Decimal;
use std::collections::HashMap;
use std::fmt;

/// Result of validating a transaction list.
#[derive(Debug, Clone, Default)]
pub struct ValidationResult {
    /// Critical errors that prevent calculation.
    pub errors: Vec<ValidationError>,
    /// Warnings that don't prevent calculation but may indicate issues.
    pub warnings: Vec<ValidationWarning>,
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
#[derive(Debug, Clone)]
pub struct ValidationError {
    /// Line number in original input (if known).
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
#[derive(Debug, Clone)]
pub struct ValidationWarning {
    /// Line number in original input (if known).
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
                expenses,
            } => {
                // Check zero quantity
                if *amount == Decimal::ZERO {
                    result.errors.push(ValidationError {
                        line,
                        date: tx.date,
                        ticker: tx.ticker.clone(),
                        message: "BUY with zero quantity".to_string(),
                    });
                }

                // Check negative quantity
                if *amount < Decimal::ZERO {
                    result.errors.push(ValidationError {
                        line,
                        date: tx.date,
                        ticker: tx.ticker.clone(),
                        message: format!("BUY with negative quantity: {}", amount),
                    });
                }

                // Check negative price
                if *price < Decimal::ZERO {
                    result.errors.push(ValidationError {
                        line,
                        date: tx.date,
                        ticker: tx.ticker.clone(),
                        message: format!("BUY with negative price: {}", price),
                    });
                }

                // Check negative expenses
                if *expenses < Decimal::ZERO {
                    result.errors.push(ValidationError {
                        line,
                        date: tx.date,
                        ticker: tx.ticker.clone(),
                        message: format!("BUY with negative expenses: {}", expenses),
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
                expenses,
            } => {
                // Check zero quantity
                if *amount == Decimal::ZERO {
                    result.errors.push(ValidationError {
                        line,
                        date: tx.date,
                        ticker: tx.ticker.clone(),
                        message: "SELL with zero quantity".to_string(),
                    });
                }

                // Check negative quantity
                if *amount < Decimal::ZERO {
                    result.errors.push(ValidationError {
                        line,
                        date: tx.date,
                        ticker: tx.ticker.clone(),
                        message: format!("SELL with negative quantity: {}", amount),
                    });
                }

                // Check negative price
                if *price < Decimal::ZERO {
                    result.errors.push(ValidationError {
                        line,
                        date: tx.date,
                        ticker: tx.ticker.clone(),
                        message: format!("SELL with negative price: {}", price),
                    });
                }

                // Check negative expenses
                if *expenses < Decimal::ZERO {
                    result.errors.push(ValidationError {
                        line,
                        date: tx.date,
                        ticker: tx.ticker.clone(),
                        message: format!("SELL with negative expenses: {}", expenses),
                    });
                }

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
                // Check zero ratio
                if *ratio == Decimal::ZERO {
                    result.errors.push(ValidationError {
                        line,
                        date: tx.date,
                        ticker: tx.ticker.clone(),
                        message: "SPLIT with zero ratio".to_string(),
                    });
                }

                // Check negative ratio
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
                // Check zero ratio
                if *ratio == Decimal::ZERO {
                    result.errors.push(ValidationError {
                        line,
                        date: tx.date,
                        ticker: tx.ticker.clone(),
                        message: "UNSPLIT with zero ratio".to_string(),
                    });
                }

                // Check negative ratio
                if *ratio < Decimal::ZERO {
                    result.errors.push(ValidationError {
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
                    result.errors.push(ValidationError {
                        line,
                        date: tx.date,
                        ticker: tx.ticker.clone(),
                        message: "DIVIDEND with zero quantity".to_string(),
                    });
                }

                // Check negative quantity
                if *amount < Decimal::ZERO {
                    result.errors.push(ValidationError {
                        line,
                        date: tx.date,
                        ticker: tx.ticker.clone(),
                        message: format!("DIVIDEND with negative quantity: {}", amount),
                    });
                }

                // Check negative total value
                if *total_value < Decimal::ZERO {
                    result.errors.push(ValidationError {
                        line,
                        date: tx.date,
                        ticker: tx.ticker.clone(),
                        message: format!("DIVIDEND with negative total value: {}", total_value),
                    });
                }
            }

            Operation::CapReturn {
                amount,
                total_value,
                expenses,
            } => {
                // Check zero quantity
                if *amount == Decimal::ZERO {
                    result.errors.push(ValidationError {
                        line,
                        date: tx.date,
                        ticker: tx.ticker.clone(),
                        message: "CAPRETURN with zero quantity".to_string(),
                    });
                }

                // Check negative quantity
                if *amount < Decimal::ZERO {
                    result.errors.push(ValidationError {
                        line,
                        date: tx.date,
                        ticker: tx.ticker.clone(),
                        message: format!("CAPRETURN with negative quantity: {}", amount),
                    });
                }

                // Check negative total value
                if *total_value < Decimal::ZERO {
                    result.errors.push(ValidationError {
                        line,
                        date: tx.date,
                        ticker: tx.ticker.clone(),
                        message: format!("CAPRETURN with negative total value: {}", total_value),
                    });
                }

                // Check negative expenses
                if *expenses < Decimal::ZERO {
                    result.errors.push(ValidationError {
                        line,
                        date: tx.date,
                        ticker: tx.ticker.clone(),
                        message: format!("CAPRETURN with negative expenses: {}", expenses),
                    });
                }
            }
        }
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_buy(date: &str, ticker: &str, amount: i64, price: i64, expenses: i64) -> Transaction {
        Transaction {
            date: date.parse().unwrap(),
            ticker: ticker.to_string(),
            operation: Operation::Buy {
                amount: Decimal::from(amount),
                price: Decimal::from(price),
                expenses: Decimal::from(expenses),
            },
        }
    }

    fn make_sell(date: &str, ticker: &str, amount: i64, price: i64, expenses: i64) -> Transaction {
        Transaction {
            date: date.parse().unwrap(),
            ticker: ticker.to_string(),
            operation: Operation::Sell {
                amount: Decimal::from(amount),
                price: Decimal::from(price),
                expenses: Decimal::from(expenses),
            },
        }
    }

    fn make_split(date: &str, ticker: &str, ratio: i64) -> Transaction {
        Transaction {
            date: date.parse().unwrap(),
            ticker: ticker.to_string(),
            operation: Operation::Split {
                ratio: Decimal::from(ratio),
            },
        }
    }

    #[test]
    fn test_valid_transactions() {
        let txns = vec![
            make_buy("2020-01-01", "AAPL", 100, 150, 10),
            make_sell("2020-06-01", "AAPL", 50, 180, 10),
        ];
        let result = validate(&txns);
        assert!(result.is_valid());
        assert!(result.is_clean());
    }

    #[test]
    fn test_zero_quantity_buy() {
        let txns = vec![make_buy("2020-01-01", "AAPL", 0, 150, 10)];
        let result = validate(&txns);
        assert!(!result.is_valid());
        assert_eq!(result.errors.len(), 1);
        assert!(result.errors[0].message.contains("zero quantity"));
    }

    #[test]
    fn test_zero_quantity_sell() {
        let txns = vec![
            make_buy("2020-01-01", "AAPL", 100, 150, 10),
            make_sell("2020-06-01", "AAPL", 0, 180, 10),
        ];
        let result = validate(&txns);
        assert!(!result.is_valid());
        assert_eq!(result.errors.len(), 1);
        assert!(result.errors[0].message.contains("zero quantity"));
    }

    #[test]
    fn test_negative_price() {
        let txns = vec![make_buy("2020-01-01", "AAPL", 100, -150, 10)];
        let result = validate(&txns);
        assert!(!result.is_valid());
        assert!(result.errors[0].message.contains("negative price"));
    }

    #[test]
    fn test_negative_expenses() {
        let txns = vec![make_buy("2020-01-01", "AAPL", 100, 150, -10)];
        let result = validate(&txns);
        assert!(!result.is_valid());
        assert!(result.errors[0].message.contains("negative expenses"));
    }

    #[test]
    fn test_zero_split_ratio() {
        let txns = vec![make_split("2020-01-01", "AAPL", 0)];
        let result = validate(&txns);
        assert!(!result.is_valid());
        assert!(result.errors[0].message.contains("zero ratio"));
    }

    #[test]
    fn test_negative_split_ratio() {
        let txns = vec![make_split("2020-01-01", "AAPL", -2)];
        let result = validate(&txns);
        assert!(!result.is_valid());
        assert!(result.errors[0].message.contains("negative ratio"));
    }

    #[test]
    fn test_sell_before_buy_warning() {
        let txns = vec![
            make_sell("2020-01-01", "AAPL", 50, 180, 10),
            make_buy("2020-06-01", "AAPL", 100, 150, 10),
        ];
        let result = validate(&txns);
        // Valid (just a warning)
        assert!(result.is_valid());
        // But not clean
        assert!(!result.is_clean());
        assert_eq!(result.warnings.len(), 1);
        assert!(result.warnings[0].message.contains("no prior BUY"));
    }

    #[test]
    fn test_sell_with_no_buy() {
        let txns = vec![make_sell("2020-01-01", "AAPL", 50, 180, 10)];
        let result = validate(&txns);
        assert!(result.is_valid());
        assert_eq!(result.warnings.len(), 1);
        assert!(result.warnings[0].message.contains("no prior BUY"));
    }
}
