mod awards;
mod transactions;

use crate::error::ConvertError;
use crate::output;
use crate::{BrokerConverter, ConvertOutput};
use chrono::NaiveDate;
use rust_decimal::Decimal;
use std::collections::HashMap;

pub use awards::{AwardLookup, AwardsData};
use transactions::{
    SchwabDividend, SchwabStockPlanActivity, SchwabStockSplit, SchwabTrade, SchwabTransaction,
    SchwabTransactionsItem, format_unknown_comment, parse_transactions_json,
};

/// Input for Schwab converter
#[derive(Debug, Clone)]
pub struct SchwabInput {
    /// Transactions JSON content
    pub transactions_json: String,
    /// Optional equity awards JSON content
    pub awards_json: Option<String>,
}

/// Schwab converter implementation
#[derive(Debug, Default)]
pub struct SchwabConverter;

impl SchwabConverter {
    pub fn new() -> Self {
        Self
    }
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
        let awards = if let Some(ref awards_json) = input.awards_json {
            Some(awards::parse_awards_json(awards_json)?)
        } else {
            None
        };

        // Parse transactions JSON
        let transactions = parse_transactions_json(&input.transactions_json)?;

        // Convert to CGT transactions
        let (cgt_transactions, warnings, skipped_count) =
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
        let source_files = if input.awards_json.is_some() {
            vec!["transactions.json".to_string(), "awards.json".to_string()]
        } else {
            vec!["transactions.json".to_string()]
        };
        output_lines.push(output::generate_header(
            "Charles Schwab",
            &source_files,
            skipped_count,
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
            warnings,
            skipped_count,
        })
    }

    fn broker_name(&self) -> &'static str {
        "Charles Schwab"
    }
}

/// A pending cancellation from a Cancel Sell transaction.
struct PendingCancellation {
    date: NaiveDate,
    symbol: String,
    quantity: Decimal,
    price: Decimal,
}

/// Remove sells that were cancelled. Each cancellation matches a sell with
/// the same (date, symbol, quantity, price). Unmatched cancellations produce
/// warnings.
fn apply_cancellations(
    transactions: &mut Vec<CgtTransaction>,
    cancellations: Vec<PendingCancellation>,
    warnings: &mut Vec<String>,
) {
    for cancel in cancellations {
        if let Some(pos) = transactions.iter().position(|txn| {
            matches!(
                txn,
                CgtTransaction::Sell {
                    date: d,
                    symbol: s,
                    quantity: q,
                    price: p,
                    ..
                } if *d == cancel.date && s == &cancel.symbol
                    && *q == cancel.quantity && *p == cancel.price
            )
        }) {
            transactions.remove(pos);
        } else {
            warnings.push(format!(
                "Cancel Sell on {} for {} {} shares @ {} has no matching sell to cancel",
                cancel.date, cancel.symbol, cancel.quantity, cancel.price
            ));
        }
    }
}

/// Process Schwab transactions into CGT transactions
fn process_transactions(
    transactions: Vec<SchwabTransactionsItem>,
    awards: Option<&AwardsData>,
) -> Result<(Vec<CgtTransaction>, Vec<String>, usize), ConvertError> {
    let mut cgt_transactions = Vec::new();
    let mut warnings = Vec::new();
    let mut skipped_count = 0;
    let mut dividend_taxes: HashMap<(NaiveDate, String), Decimal> = HashMap::new();
    let mut pending_cancellations: Vec<PendingCancellation> = Vec::new();

    let has_stock_plan_activity = transactions.iter().any(|txn| {
        matches!(
            txn,
            SchwabTransactionsItem::Known(SchwabTransaction::StockPlanActivity(_))
        )
    });

    if awards.is_none() && has_stock_plan_activity {
        warnings.push(
            "No awards file provided; RSU vesting entries require awards data for FMV.".to_string(),
        );
    }

    // First pass: collect tax withholdings
    for txn in &transactions {
        match txn {
            SchwabTransactionsItem::Known(SchwabTransaction::NraTaxAdj(tax))
            | SchwabTransactionsItem::Known(SchwabTransaction::NraWithholding(tax)) => {
                if let (Some(symbol), Some(tax_amount)) = (tax.symbol.as_ref(), tax.amount) {
                    let key = (tax.date, symbol.clone());
                    *dividend_taxes.entry(key).or_insert(Decimal::ZERO) += tax_amount.abs();
                }
            }
            _ => {}
        }
    }

    // Second pass: convert transactions
    for record in transactions {
        match record {
            SchwabTransactionsItem::Known(txn) => match txn {
                SchwabTransaction::Buy(trade) => {
                    let SchwabTrade {
                        common,
                        quantity,
                        price,
                        fees_commissions,
                    } = trade;
                    cgt_transactions.push(CgtTransaction::Buy {
                        date: common.date,
                        symbol: common.symbol,
                        quantity,
                        price,
                        expenses: fees_commissions.unwrap_or(Decimal::ZERO),
                        comment: None,
                    });
                }
                SchwabTransaction::Sell(trade) => {
                    let SchwabTrade {
                        common,
                        quantity,
                        price,
                        fees_commissions,
                    } = trade;
                    cgt_transactions.push(CgtTransaction::Sell {
                        date: common.date,
                        symbol: common.symbol,
                        quantity,
                        price,
                        expenses: fees_commissions.unwrap_or(Decimal::ZERO),
                    });
                }
                SchwabTransaction::CancelSell(trade) => {
                    // Cancel Sell reverses a prior sell (e.g., Schwab price correction).
                    // The cancelled sell never happened — collect for post-processing
                    // since the original sell may appear later in the JSON (Schwab
                    // lists newest transactions first).
                    pending_cancellations.push(PendingCancellation {
                        date: trade.common.date,
                        symbol: trade.common.symbol,
                        quantity: trade.quantity,
                        price: trade.price,
                    });
                }
                SchwabTransaction::StockPlanActivity(activity) => {
                    let SchwabStockPlanActivity { common, quantity } = activity;
                    // RSU vesting - need FMV and vest date from awards file
                    // Per HMRC guidance (CG14250, ERSM20192), acquisition date is the vest date
                    // (when conditions are satisfied), not the settlement date from transactions
                    let award_lookup = if let Some(awards_data) = awards {
                        awards_data.get_fmv(&common.date, &common.symbol)?
                    } else {
                        return Err(ConvertError::MissingFairMarketValue {
                            date: common.date.to_string(),
                            symbol: common.symbol.clone(),
                        });
                    };

                    cgt_transactions.push(CgtTransaction::Buy {
                        // Use vest date from awards file as CGT acquisition date
                        date: award_lookup.vest_date,
                        symbol: common.symbol,
                        quantity,
                        price: award_lookup.fmv,
                        expenses: Decimal::ZERO,
                        comment: Some("RSU Vesting - FMV from awards file".to_string()),
                    });
                }
                SchwabTransaction::CashDividend(dividend)
                | SchwabTransaction::QualifiedDividend(dividend)
                | SchwabTransaction::ShortTermCapGain(dividend)
                | SchwabTransaction::LongTermCapGain(dividend) => {
                    let SchwabDividend { common, amount } = dividend;
                    if let Some(amount) = amount {
                        let amount_value = amount.abs();
                        let key = (common.date, common.symbol.clone());
                        let tax = dividend_taxes.remove(&key).unwrap_or(Decimal::ZERO);

                        cgt_transactions.push(CgtTransaction::Dividend {
                            date: common.date,
                            symbol: common.symbol,
                            amount: amount_value,
                            tax,
                        });
                    }
                }
                SchwabTransaction::StockSplit(split) => {
                    let SchwabStockSplit { common } = split;
                    // Note: Schwab doesn't provide split ratio directly
                    // Add as comment for user to fill in manually
                    let comment = format!(
                        "UNSUPPORTED: Stock split for {} on {} - please add SPLIT transaction manually with correct ratio",
                        common.symbol,
                        common.date.format("%Y-%m-%d")
                    );
                    // We add it to cgt_transactions so it appears in the output, but it doesn't count as a "real" transaction logic-wise
                    cgt_transactions.push(CgtTransaction::Comment {
                        comment: comment.clone(),
                    });
                    skipped_count += 1;
                }
                SchwabTransaction::NraTaxAdj(_) | SchwabTransaction::NraWithholding(_) => {
                    // Already processed in first pass
                }
                SchwabTransaction::NonCgt => {
                    skipped_count += 1;
                }
            },
            SchwabTransactionsItem::Unknown(raw) => {
                let comment = format_unknown_comment(&raw);
                warnings.push(format!(
                    "Unknown Schwab action '{}' — skipped. \
                     Please report this so it can be added to the converter.",
                    comment
                ));
                cgt_transactions.push(CgtTransaction::Comment {
                    comment: comment.clone(),
                });
                skipped_count += 1;
            }
        }
    }

    // Apply deferred cancellations: remove original sells that were cancelled.
    // This must happen after all transactions are processed because Cancel Sell
    // entries can appear before their corresponding original Sell in the JSON.
    if !pending_cancellations.is_empty() {
        apply_cancellations(&mut cgt_transactions, pending_cancellations, &mut warnings);
    }

    Ok((cgt_transactions, warnings, skipped_count))
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_unknown_transaction_produces_comment() {
        // Confirms unknown transactions become comments and are recorded as skipped
        let json = r#"{
            "BrokerageTransactions": [
                {
                    "Date": "04/22/2021",
                    "Action": "UnknownAction",
                    "Symbol": "XYZ",
                    "Description": "Unknown transaction",
                    "Quantity": "",
                    "Price": "",
                    "Fees & Comm": "",
                    "Amount": "-$43,640.34"
                }
            ]
        }"#;

        let input = SchwabInput {
            transactions_json: json.to_string(),
            awards_json: None,
        };

        let converter = SchwabConverter::new();
        let result = converter.convert(&input).unwrap();

        assert_eq!(result.skipped_count, 1);
        // Unknown actions produce a warning
        assert_eq!(result.warnings.len(), 1);
        assert!(result.warnings[0].contains("Unknown Schwab action"));
        assert!(
            result
                .cgt_content
                .contains("# SKIPPED: UnknownAction - XYZ on 2021-04-22 (Unknown transaction)")
        );
    }
}
