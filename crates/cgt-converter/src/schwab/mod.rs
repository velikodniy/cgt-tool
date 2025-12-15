mod awards;

use crate::error::ConvertError;
use crate::output;
use crate::{BrokerConverter, ConvertOutput};
use chrono::NaiveDate;
use csv::StringRecord;
use rust_decimal::Decimal;
use std::collections::HashMap;
use std::str::FromStr;

pub use awards::{AwardLookup, AwardsData, AwardsFormat};

/// Input for Schwab converter
#[derive(Debug, Clone)]
pub struct SchwabInput {
    /// Transactions CSV content
    pub transactions_csv: String,
    /// Optional equity awards content
    pub awards_content: Option<String>,
    /// Format of awards file (if provided)
    pub awards_format: Option<awards::AwardsFormat>,
}

/// Schwab converter implementation
#[derive(Debug, Default)]
pub struct SchwabConverter;

impl SchwabConverter {
    pub fn new() -> Self {
        Self
    }
}

/// Internal representation of a Schwab transaction
#[derive(Debug, Clone)]
struct SchwabTransaction {
    date: NaiveDate,
    action: String,
    symbol: String,
    description: String,
    quantity: Option<Decimal>,
    price: Option<Decimal>,
    fees_commissions: Option<Decimal>,
    amount: Option<Decimal>,
}

/// CGT-relevant transaction after processing
#[derive(Debug, Clone)]
enum CgtTransaction {
    Buy {
        date: NaiveDate,
        symbol: String,
        quantity: Decimal,
        price: Decimal,
        expenses: Decimal,
        comment: Option<String>,
    },
    Sell {
        date: NaiveDate,
        symbol: String,
        quantity: Decimal,
        price: Decimal,
        expenses: Decimal,
    },
    Dividend {
        date: NaiveDate,
        symbol: String,
        amount: Decimal,
        tax: Decimal,
    },
    Comment {
        comment: String,
    },
}

impl BrokerConverter for SchwabConverter {
    type Input = SchwabInput;

    fn convert(&self, input: &Self::Input) -> Result<ConvertOutput, ConvertError> {
        // Parse awards data if provided
        let awards = if let Some(ref awards_content) = input.awards_content {
            let format = input.awards_format.as_ref().ok_or_else(|| {
                ConvertError::InvalidTransaction(
                    "Awards content provided but format is missing".into(),
                )
            })?;
            Some(awards::parse_awards(awards_content, *format)?)
        } else {
            None
        };

        // Parse transactions CSV
        let transactions = parse_transactions_csv(&input.transactions_csv)?;

        // Convert to CGT transactions
        let (cgt_transactions, skipped_warnings) =
            process_transactions(transactions, awards.as_ref())?;

        // Sort chronologically (oldest first)
        // Comments don't have dates, so they stay in insertion order
        let mut sorted_txns = cgt_transactions;
        sorted_txns.sort_by_key(|txn| match txn {
            CgtTransaction::Buy { date, .. }
            | CgtTransaction::Sell { date, .. }
            | CgtTransaction::Dividend { date, .. } => Some(*date),
            CgtTransaction::Comment { .. } => None,
        });

        // Generate CGT DSL output
        let mut output_lines = Vec::new();

        // Add header
        let source_files = if input.awards_content.is_some() {
            let awards_filename = match input.awards_format {
                Some(awards::AwardsFormat::Json) => "awards.json",
                Some(awards::AwardsFormat::Csv) => "awards.csv",
                None => "awards",
            };
            vec!["transactions.csv".to_string(), awards_filename.to_string()]
        } else {
            vec!["transactions.csv".to_string()]
        };
        output_lines.push(output::generate_header(
            "Charles Schwab",
            &source_files,
            &skipped_warnings,
        ));

        // Add transactions
        for txn in &sorted_txns {
            match txn {
                CgtTransaction::Buy {
                    date,
                    symbol,
                    quantity,
                    price,
                    expenses,
                    comment,
                } => {
                    if let Some(c) = comment {
                        output_lines.push(output::format_comment(c));
                    }
                    output_lines.push(output::format_buy(
                        date,
                        symbol,
                        *quantity,
                        *price,
                        "USD",
                        Some(*expenses),
                    ));
                }
                CgtTransaction::Sell {
                    date,
                    symbol,
                    quantity,
                    price,
                    expenses,
                } => {
                    output_lines.push(output::format_sell(
                        date,
                        symbol,
                        *quantity,
                        *price,
                        "USD",
                        Some(*expenses),
                    ));
                }
                CgtTransaction::Dividend {
                    date,
                    symbol,
                    amount,
                    tax,
                } => {
                    output_lines.push(output::format_dividend(
                        date,
                        symbol,
                        *amount,
                        "USD",
                        Some(*tax),
                    ));
                }
                CgtTransaction::Comment { comment } => {
                    output_lines.push(output::format_comment(comment));
                }
            }
        }

        Ok(ConvertOutput {
            cgt_content: output_lines.join("\n"),
            warnings: skipped_warnings.clone(),
            skipped_count: skipped_warnings.len(),
        })
    }

    fn broker_name(&self) -> &'static str {
        "Charles Schwab"
    }
}

/// Parse Schwab transactions CSV
fn parse_transactions_csv(csv_content: &str) -> Result<Vec<SchwabTransaction>, ConvertError> {
    let mut reader = csv::ReaderBuilder::new()
        .has_headers(true)
        .flexible(true) // Allow variable number of fields per record
        .from_reader(csv_content.as_bytes());

    let headers = reader.headers()?.clone();
    let mut transactions = Vec::new();

    for result in reader.records() {
        let record = result?;
        let txn = parse_transaction_row(&headers, &record)?;
        transactions.push(txn);
    }

    Ok(transactions)
}

/// Parse a single transaction row
fn parse_transaction_row(
    headers: &StringRecord,
    record: &StringRecord,
) -> Result<SchwabTransaction, ConvertError> {
    let get_field = |name: &str| -> Result<&str, ConvertError> {
        headers
            .iter()
            .position(|h| h == name)
            .and_then(|idx| record.get(idx))
            .ok_or_else(|| ConvertError::MissingColumn(name.to_string()))
    };

    let date_str = get_field("Date")?;
    let date = parse_date(date_str)?;

    let action = get_field("Action")?.to_string();
    let symbol = get_field("Symbol")?.trim().to_string();
    let description = get_field("Description")?.to_string();

    // Optional numeric fields
    let quantity = get_field("Quantity")
        .ok()
        .and_then(|s| parse_amount(s).ok())
        .flatten();

    let price = get_field("Price")
        .ok()
        .and_then(|s| parse_amount(s).ok())
        .flatten();

    let fees_commissions = get_field("Fees & Comm")
        .ok()
        .and_then(|s| parse_amount(s).ok())
        .flatten();

    let amount = get_field("Amount")
        .ok()
        .and_then(|s| parse_amount(s).ok())
        .flatten();

    Ok(SchwabTransaction {
        date,
        action,
        symbol,
        description,
        quantity,
        price,
        fees_commissions,
        amount,
    })
}

/// Parse a Schwab date (MM/DD/YYYY format, with "as of" handling)
fn parse_date(date_str: &str) -> Result<NaiveDate, ConvertError> {
    let clean_date = date_str.trim();

    // Handle "MM/DD/YYYY as of MM/DD/YYYY" format - use the second (actual) date
    if let Some(as_of_pos) = clean_date.find(" as of ") {
        let actual_date = &clean_date[as_of_pos + 7..]; // Skip " as of " (7 chars)
        NaiveDate::parse_from_str(actual_date.trim(), "%m/%d/%Y")
            .map_err(|_| ConvertError::InvalidDate(date_str.to_string()))
    } else if let Some(date_part) = clean_date.strip_prefix("as of ") {
        // Handle "as of MM/DD/YYYY" format (prefix)
        NaiveDate::parse_from_str(date_part, "%m/%d/%Y")
            .map_err(|_| ConvertError::InvalidDate(date_str.to_string()))
    } else {
        // Parse MM/DD/YYYY format
        NaiveDate::parse_from_str(clean_date, "%m/%d/%Y")
            .map_err(|_| ConvertError::InvalidDate(date_str.to_string()))
    }
}

/// Parse a Schwab amount (handles $-prefix, commas, empty strings)
fn parse_amount(amount_str: &str) -> Result<Option<Decimal>, ConvertError> {
    let trimmed = amount_str.trim();
    if trimmed.is_empty() || trimmed == "--" {
        return Ok(None);
    }

    // Remove $ prefix and commas
    let cleaned = trimmed.replace(['$', ','], "");

    Decimal::from_str(&cleaned)
        .map(Some)
        .map_err(|_| ConvertError::InvalidAmount(amount_str.to_string()))
}

/// Process Schwab transactions into CGT transactions
fn process_transactions(
    transactions: Vec<SchwabTransaction>,
    awards: Option<&AwardsData>,
) -> Result<(Vec<CgtTransaction>, Vec<String>), ConvertError> {
    let mut cgt_transactions = Vec::new();
    let mut warnings = Vec::new();
    let mut dividend_taxes: HashMap<(NaiveDate, String), Decimal> = HashMap::new();

    // First pass: collect tax withholdings
    for txn in &transactions {
        if (txn.action == "NRA Tax Adj" || txn.action == "NRA Withholding")
            && let Some(amount) = txn.amount
        {
            let tax_amount = amount.abs();
            let key = (txn.date, txn.symbol.clone());
            *dividend_taxes.entry(key).or_insert(Decimal::ZERO) += tax_amount;
        }
    }

    // Second pass: convert transactions
    for txn in transactions {
        match txn.action.as_str() {
            "Buy" => {
                let quantity = txn.quantity.ok_or_else(|| {
                    ConvertError::InvalidTransaction("Buy missing quantity".into())
                })?;
                let price = txn
                    .price
                    .ok_or_else(|| ConvertError::InvalidTransaction("Buy missing price".into()))?;
                let expenses = txn.fees_commissions.unwrap_or(Decimal::ZERO);

                cgt_transactions.push(CgtTransaction::Buy {
                    date: txn.date,
                    symbol: txn.symbol,
                    quantity,
                    price,
                    expenses,
                    comment: None,
                });
            }
            "Sell" => {
                let quantity = txn.quantity.ok_or_else(|| {
                    ConvertError::InvalidTransaction("Sell missing quantity".into())
                })?;
                let price = txn
                    .price
                    .ok_or_else(|| ConvertError::InvalidTransaction("Sell missing price".into()))?;
                let expenses = txn.fees_commissions.unwrap_or(Decimal::ZERO);

                cgt_transactions.push(CgtTransaction::Sell {
                    date: txn.date,
                    symbol: txn.symbol,
                    quantity,
                    price,
                    expenses,
                });
            }
            "Stock Plan Activity" => {
                // RSU vesting - need FMV and vest date from awards file
                // Per HMRC guidance (CG14250, ERSM20192), acquisition date is the vest date
                // (when conditions are satisfied), not the settlement date from transactions
                let quantity = txn.quantity.ok_or_else(|| {
                    ConvertError::InvalidTransaction("Stock Plan Activity missing quantity".into())
                })?;

                let award_lookup = if let Some(awards_data) = awards {
                    awards_data.get_fmv(&txn.date, &txn.symbol)?
                } else {
                    return Err(ConvertError::MissingFairMarketValue {
                        date: txn.date.to_string(),
                        symbol: txn.symbol.clone(),
                    });
                };

                cgt_transactions.push(CgtTransaction::Buy {
                    // Use vest date from awards file as CGT acquisition date
                    date: award_lookup.vest_date,
                    symbol: txn.symbol,
                    quantity,
                    price: award_lookup.fmv,
                    expenses: Decimal::ZERO,
                    comment: Some("RSU Vesting - FMV from awards file".to_string()),
                });
            }
            "Cash Dividend"
            | "Qualified Dividend"
            | "Short Term Cap Gain"
            | "Long Term Cap Gain" => {
                if let Some(amount) = txn.amount {
                    let amount_value = amount.abs();
                    let key = (txn.date, txn.symbol.clone());
                    let tax = dividend_taxes.remove(&key).unwrap_or(Decimal::ZERO);

                    cgt_transactions.push(CgtTransaction::Dividend {
                        date: txn.date,
                        symbol: txn.symbol,
                        amount: amount_value,
                        tax,
                    });
                }
            }
            "Stock Split" => {
                // Note: Schwab doesn't provide split ratio directly
                // Add as comment for user to fill in manually
                let comment = format!(
                    "UNSUPPORTED: Stock split for {} on {} - please add SPLIT transaction manually with correct ratio",
                    txn.symbol,
                    txn.date.format("%Y-%m-%d")
                );
                cgt_transactions.push(CgtTransaction::Comment {
                    comment: comment.clone(),
                });
                warnings.push(comment);
            }
            "NRA Tax Adj" | "NRA Withholding" => {
                // Already processed in first pass
            }
            _ => {
                // Add unsupported transaction as comment in the output
                let comment = format!(
                    "SKIPPED: {} - {} on {} ({})",
                    txn.action,
                    txn.symbol,
                    txn.date.format("%Y-%m-%d"),
                    txn.description
                );
                cgt_transactions.push(CgtTransaction::Comment {
                    comment: comment.clone(),
                });
                warnings.push(comment);
            }
        }
    }

    Ok((cgt_transactions, warnings))
}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal_macros::dec;

    #[test]
    fn test_parse_date_standard() {
        let result = parse_date("04/25/2023").unwrap();
        assert_eq!(result, NaiveDate::from_ymd_opt(2023, 4, 25).unwrap());
    }

    #[test]
    fn test_parse_date_as_of() {
        let result = parse_date("as of 04/25/2023").unwrap();
        assert_eq!(result, NaiveDate::from_ymd_opt(2023, 4, 25).unwrap());
    }

    #[test]
    fn test_parse_date_with_as_of_suffix() {
        let result = parse_date("03/20/2024 as of 03/19/2024").unwrap();
        // Should use the actual date (after "as of")
        assert_eq!(result, NaiveDate::from_ymd_opt(2024, 3, 19).unwrap());
    }

    #[test]
    fn test_parse_amount_with_dollar() {
        let result = parse_amount("$125.64").unwrap().unwrap();
        assert_eq!(result, dec!(125.64));
    }

    #[test]
    fn test_parse_amount_with_commas() {
        let result = parse_amount("$1,234.56").unwrap().unwrap();
        assert_eq!(result, dec!(1234.56));
    }

    #[test]
    fn test_parse_amount_empty() {
        let result = parse_amount("").unwrap();
        assert_eq!(result, None);
    }

    #[test]
    fn test_parse_amount_dashes() {
        let result = parse_amount("--").unwrap();
        assert_eq!(result, None);
    }

    // ===========================================
    // Date Format Edge Cases
    // ===========================================

    #[test]
    fn test_parse_date_with_as_of_single_digit_month() {
        // Single-digit month in "as of" format (e.g., 1/17/2023 instead of 01/17/2023)
        let result = parse_date("1/18/2023 as of 1/17/2023").unwrap();
        assert_eq!(result, NaiveDate::from_ymd_opt(2023, 1, 17).unwrap());
    }

    #[test]
    fn test_parse_date_with_as_of_single_digit_day() {
        // Single-digit day
        let result = parse_date("10/5/2023 as of 10/4/2023").unwrap();
        assert_eq!(result, NaiveDate::from_ymd_opt(2023, 10, 4).unwrap());
    }

    #[test]
    fn test_parse_date_with_whitespace() {
        let result = parse_date("  04/25/2023  ").unwrap();
        assert_eq!(result, NaiveDate::from_ymd_opt(2023, 4, 25).unwrap());
    }

    #[test]
    fn test_parse_date_cross_year_as_of() {
        // Date in January "as of" December previous year
        let result = parse_date("01/02/2024 as of 12/31/2023").unwrap();
        assert_eq!(result, NaiveDate::from_ymd_opt(2023, 12, 31).unwrap());
    }

    #[test]
    fn test_parse_date_leap_year() {
        let result = parse_date("02/29/2024").unwrap();
        assert_eq!(result, NaiveDate::from_ymd_opt(2024, 2, 29).unwrap());
    }

    #[test]
    fn test_parse_date_invalid_format_returns_error() {
        // ISO format should fail
        assert!(parse_date("2023-04-25").is_err());
        // European format should fail
        assert!(parse_date("25/04/2023").is_err());
        // Invalid date
        assert!(parse_date("13/45/2023").is_err());
        // Empty string
        assert!(parse_date("").is_err());
    }

    // ===========================================
    // Amount Parsing Edge Cases
    // ===========================================

    #[test]
    fn test_parse_amount_negative() {
        let result = parse_amount("-$125.64").unwrap().unwrap();
        assert_eq!(result, dec!(-125.64));
    }

    #[test]
    fn test_parse_amount_large_number() {
        let result = parse_amount("$1,234,567.89").unwrap().unwrap();
        assert_eq!(result, dec!(1234567.89));
    }

    #[test]
    fn test_parse_amount_zero() {
        let result = parse_amount("$0.00").unwrap().unwrap();
        assert_eq!(result, dec!(0.00));
    }

    #[test]
    fn test_parse_amount_small_decimal() {
        let result = parse_amount("$0.01").unwrap().unwrap();
        assert_eq!(result, dec!(0.01));
    }

    #[test]
    fn test_parse_amount_many_decimals() {
        let result = parse_amount("$125.6445").unwrap().unwrap();
        assert_eq!(result, dec!(125.6445));
    }

    #[test]
    fn test_parse_amount_whitespace() {
        let result = parse_amount("  $125.64  ").unwrap().unwrap();
        assert_eq!(result, dec!(125.64));
    }

    #[test]
    fn test_parse_amount_no_dollar() {
        let result = parse_amount("125.64").unwrap().unwrap();
        assert_eq!(result, dec!(125.64));
    }
}
