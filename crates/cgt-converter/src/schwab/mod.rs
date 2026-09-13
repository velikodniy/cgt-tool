mod awards;
mod transactions;

use crate::error::ConvertError;
use crate::output;
use crate::{BrokerConverter, ConvertOutput};
use cgt::money::{Currency, CurrencyAmount};
use cgt::{Operation, Transaction};
use chrono::NaiveDate;
use rust_decimal::Decimal;
use std::collections::BTreeMap;

pub use awards::{AwardLookup, AwardsData};

/// Parse a Schwab dollar amount string: strips `$`, commas, handles empty/placeholder values.
pub(crate) fn parse_dollar_amount(s: &str) -> Result<Option<Decimal>, ConvertError> {
    let trimmed = s.trim();
    if trimmed.is_empty() || trimmed == "--" {
        return Ok(None);
    }
    let cleaned = trimmed.replace(['$', ','], "");
    cleaned
        .parse::<Decimal>()
        .map(Some)
        .map_err(|_| ConvertError::InvalidAmount(s.to_string()))
}
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

struct AnnotatedTransaction {
    transaction: Transaction,
    comment: Option<String>,
}

impl AnnotatedTransaction {
    fn new(date: NaiveDate, ticker: String, operation: Operation<CurrencyAmount>) -> Self {
        Self {
            transaction: Transaction {
                date,
                ticker,
                operation,
                line: None,
            },
            comment: None,
        }
    }
}

struct ConvertedTransactions {
    transactions: Vec<AnnotatedTransaction>,
    comments: Vec<String>,
    warnings: Vec<String>,
    skipped_count: usize,
}

fn usd(amount: Decimal) -> CurrencyAmount {
    CurrencyAmount::new(amount, Currency::USD)
}

// Zero optional amounts use the parser default, so serialization omits them.
fn optional_usd(amount: Decimal) -> CurrencyAmount {
    if amount.is_zero() {
        CurrencyAmount::new(amount, Currency::GBP)
    } else {
        usd(amount)
    }
}

impl BrokerConverter for SchwabConverter {
    type Input = SchwabInput;

    fn convert(&self, input: &Self::Input) -> Result<ConvertOutput, ConvertError> {
        let awards = input
            .awards_json
            .as_deref()
            .map(awards::parse_awards_json)
            .transpose()?;
        let transactions = parse_transactions_json(&input.transactions_json)?;
        let mut converted = process_transactions(transactions, awards.as_ref())?;
        converted
            .transactions
            .sort_by_key(|record| record.transaction.date);
        let source_files = if input.awards_json.is_some() {
            vec!["transactions.json".to_string(), "awards.json".to_string()]
        } else {
            vec!["transactions.json".to_string()]
        };
        let mut lines = vec![output::generate_header(
            "Charles Schwab",
            &source_files,
            converted.skipped_count,
        )];
        lines.extend(
            converted
                .comments
                .iter()
                .map(|comment| output::format_comment(comment)),
        );
        for record in converted.transactions {
            if let Some(comment) = record.comment {
                lines.push(output::format_comment(&comment));
            }
            lines.push(cgt::dsl::serialize_transaction(&record.transaction));
        }
        let cgt_content = lines.join("\n");
        cgt::dsl::parse(&cgt_content).map_err(|error| {
            ConvertError::InvalidTransaction(format!(
                "Converted output is not valid CGT DSL: {error}"
            ))
        })?;
        Ok(ConvertOutput {
            cgt_content,
            warnings: converted.warnings,
            skipped_count: converted.skipped_count,
        })
    }

    fn broker_name(&self) -> &'static str {
        "Charles Schwab"
    }
}

fn apply_cancellations(
    transactions: &mut Vec<AnnotatedTransaction>,
    cancellations: Vec<SchwabTrade>,
    warnings: &mut Vec<String>,
) -> Result<(), ConvertError> {
    for cancel in cancellations {
        let candidates: Vec<_> = transactions
            .iter()
            .enumerate()
            .filter_map(|(position, record)| {
                let transaction = &record.transaction;
                if let Operation::Sell {
                    amount,
                    price,
                    fees,
                } = &transaction.operation
                    && transaction.date == cancel.common.date
                    && transaction.ticker == cancel.common.symbol
                    && *amount == cancel.quantity
                    && price.amount == cancel.price
                    && cancel
                        .fees_commissions
                        .is_none_or(|cancel_fees| cancel_fees == fees.amount)
                {
                    Some((position, fees.amount))
                } else {
                    None
                }
            })
            .collect();
        if let Some(&(position, expenses)) = candidates.first() {
            if candidates
                .iter()
                .any(|(_, candidate_expenses)| *candidate_expenses != expenses)
            {
                return Err(ConvertError::InvalidTransaction(format!(
                    "Ambiguous Cancel Sell for {} on {}: matching sales have different fees",
                    cancel.common.symbol, cancel.common.date
                )));
            }
            transactions.remove(position);
        } else {
            warnings.push(format!(
                "Cancel Sell on {} for {} {} shares @ {} has no matching sell to cancel",
                cancel.common.date, cancel.common.symbol, cancel.quantity, cancel.price
            ));
        }
    }
    Ok(())
}

fn process_transactions(
    transactions: Vec<SchwabTransactionsItem>,
    awards: Option<&AwardsData>,
) -> Result<ConvertedTransactions, ConvertError> {
    let mut converted = ConvertedTransactions {
        transactions: Vec::new(),
        comments: Vec::new(),
        warnings: Vec::new(),
        skipped_count: 0,
    };
    let mut dividend_taxes: BTreeMap<(NaiveDate, String), Decimal> = BTreeMap::new();
    let mut cancellations = Vec::new();
    let mut dividend_corrections = Vec::new();

    for record in &transactions {
        if let SchwabTransactionsItem::Known(
            SchwabTransaction::NraTaxAdj(tax) | SchwabTransaction::NraWithholding(tax),
        ) = record
        {
            if let Some(symbol) = tax.symbol.as_ref() {
                let total = dividend_taxes
                    .entry((tax.date, symbol.clone()))
                    .or_insert(Decimal::ZERO);
                *total = total.checked_sub(tax.amount).ok_or_else(|| {
                    ConvertError::InvalidTransaction(format!(
                        "Tax adjustment total overflows for {symbol} on {}",
                        tax.date
                    ))
                })?;
            } else {
                converted.warnings.push(format!(
                    "Unresolved tax adjustment on {}: missing symbol for {} USD",
                    tax.date, tax.amount
                ));
            }
        }
    }

    for record in transactions {
        match record {
            SchwabTransactionsItem::Known(transaction) => match transaction {
                SchwabTransaction::Buy(trade) => {
                    converted.transactions.push(AnnotatedTransaction::new(
                        trade.common.date,
                        trade.common.symbol,
                        Operation::Buy {
                            amount: trade.quantity,
                            price: usd(trade.price),
                            fees: optional_usd(trade.fees_commissions.unwrap_or(Decimal::ZERO)),
                        },
                    ));
                }
                SchwabTransaction::Sell(trade) => {
                    converted.transactions.push(AnnotatedTransaction::new(
                        trade.common.date,
                        trade.common.symbol,
                        Operation::Sell {
                            amount: trade.quantity,
                            price: usd(trade.price),
                            fees: optional_usd(trade.fees_commissions.unwrap_or(Decimal::ZERO)),
                        },
                    ));
                }
                SchwabTransaction::CancelSell(trade) => cancellations.push(trade),
                SchwabTransaction::StockPlanActivity(SchwabStockPlanActivity {
                    common,
                    quantity,
                }) => {
                    let awards = awards.ok_or_else(|| ConvertError::MissingFairMarketValue {
                        date: common.date.to_string(),
                        symbol: common.symbol.clone(),
                    })?;
                    let award =
                        awards.get_fmv_for_deposit(&common.date, &common.symbol, quantity)?;
                    let mut transaction = AnnotatedTransaction::new(
                        award.vest_date,
                        common.symbol,
                        Operation::Buy {
                            amount: quantity,
                            price: usd(award.fmv),
                            fees: optional_usd(Decimal::ZERO),
                        },
                    );
                    transaction.comment = Some("RSU Vesting - FMV from awards file".to_string());
                    converted.transactions.push(transaction);
                }
                SchwabTransaction::CashDividend(dividend)
                | SchwabTransaction::QualifiedDividend(dividend)
                | SchwabTransaction::ShortTermCapGain(dividend)
                | SchwabTransaction::LongTermCapGain(dividend) => {
                    let SchwabDividend { common, amount } = dividend;
                    if amount.is_sign_negative() {
                        dividend_corrections.push((common.date, common.symbol, amount.abs()));
                        continue;
                    }
                    let tax = dividend_taxes
                        .remove(&(common.date, common.symbol.clone()))
                        .unwrap_or(Decimal::ZERO);
                    if tax.is_sign_negative() {
                        return Err(ConvertError::InvalidTransaction(format!(
                            "Tax credits exceed matching withholding for {} on {}",
                            common.symbol, common.date
                        )));
                    }
                    converted.transactions.push(AnnotatedTransaction::new(
                        common.date,
                        common.symbol,
                        Operation::Dividend {
                            total_value: usd(amount),
                            tax_paid: optional_usd(tax),
                        },
                    ));
                }
                SchwabTransaction::StockSplit(SchwabStockSplit { common }) => {
                    converted.comments.push(format!(
                        "UNSUPPORTED: Stock split for {} on {} - please add SPLIT transaction manually with correct ratio",
                        common.symbol, common.date
                    ));
                    converted.skipped_count += 1;
                }
                SchwabTransaction::NraTaxAdj(_) | SchwabTransaction::NraWithholding(_) => {}
                SchwabTransaction::NonCgt => converted.skipped_count += 1,
            },
            SchwabTransactionsItem::Unknown(raw) => {
                let comment = format_unknown_comment(&raw);
                converted.warnings.push(format!(
                    "Unknown Schwab action '{}' — skipped. Please report this so it can be added to the converter.", comment
                ));
                converted.comments.push(comment);
                converted.skipped_count += 1;
            }
        }
    }

    apply_cancellations(
        &mut converted.transactions,
        cancellations,
        &mut converted.warnings,
    )?;
    for (date, symbol, mut remaining) in dividend_corrections {
        for record in &mut converted.transactions {
            if let Operation::Dividend { total_value, .. } = &mut record.transaction.operation
                && record.transaction.date == date
                && record.transaction.ticker == symbol
            {
                let reduction = total_value.amount.min(remaining);
                total_value.amount -= reduction;
                remaining -= reduction;
            }
        }
        if !remaining.is_zero() {
            return Err(ConvertError::InvalidTransaction(format!(
                "Dividend correction for {symbol} on {date} exceeds the matching dividend income by {remaining} USD"
            )));
        }
    }
    for ((date, symbol), amount) in dividend_taxes {
        if !amount.is_zero() {
            converted.warnings.push(format!(
                "Unresolved tax adjustment for {symbol} on {date}: {amount} USD has no matching dividend"
            ));
        }
    }
    Ok(converted)
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
