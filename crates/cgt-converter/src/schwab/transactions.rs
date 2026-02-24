use crate::error::ConvertError;
use chrono::NaiveDate;
use rust_decimal::Decimal;
use serde::Deserialize;
use serde_json::Value;
use std::str::FromStr;

const KEY_ACTION: &str = "Action";
const KEY_DATE: &str = "Date";
const KEY_SYMBOL: &str = "Symbol";
const KEY_DESCRIPTION: &str = "Description";
const KEY_QUANTITY: &str = "Quantity";
const KEY_PRICE: &str = "Price";
const KEY_FEES: &str = "Fees & Comm";
const KEY_AMOUNT: &str = "Amount";

const LABEL_QUANTITY: &str = "quantity";
const LABEL_PRICE: &str = "price";

#[derive(Debug, Clone, Deserialize)]
struct SchwabTransactions {
    #[serde(rename = "BrokerageTransactions")]
    brokerage_transactions: Vec<Value>,
}

#[derive(Debug, Clone)]
pub(crate) enum SchwabTransactionsItem {
    Known(SchwabTransaction),
    Unknown(SchwabRawTransaction),
}

string_enum! {
    pub(crate) enum SchwabAction {
        // CGT-relevant actions
        Buy => "Buy",
        Sell => "Sell",
        CancelSell => "Cancel Sell",
        StockPlanActivity => "Stock Plan Activity",
        CashDividend => "Cash Dividend",
        QualifiedDividend => "Qualified Dividend",
        ShortTermCapGain => "Short Term Cap Gain",
        LongTermCapGain => "Long Term Cap Gain",
        StockSplit => "Stock Split",
        NraTaxAdj => "NRA Tax Adj",
        NraWithholding => "NRA Withholding",
        // Non-CGT actions (cash movements, fees, adjustments)
        Adjustment => "Adjustment",
        CreditInterest => "Credit Interest",
        Journal => "Journal",
        MiscCashEntry => "Misc Cash Entry",
        MoneyLinkTransfer => "MoneyLink Transfer",
        ServiceFee => "Service Fee",
        WireFundsAdj => "Wire Funds Adj",
        WireSent => "Wire Sent",
    }
}

#[derive(Debug, Clone)]
pub(crate) struct SchwabTrade {
    pub(crate) common: CommonFields,
    pub(crate) quantity: Decimal,
    pub(crate) price: Decimal,
    pub(crate) fees_commissions: Option<Decimal>,
}

#[derive(Debug, Clone)]
pub(crate) struct SchwabStockPlanActivity {
    pub(crate) common: CommonFields,
    pub(crate) quantity: Decimal,
}

#[derive(Debug, Clone)]
pub(crate) struct SchwabDividend {
    pub(crate) common: CommonFields,
    pub(crate) amount: Option<Decimal>,
}

#[derive(Debug, Clone)]
pub(crate) struct SchwabStockSplit {
    pub(crate) common: CommonFields,
}

#[derive(Debug, Clone)]
pub(crate) struct SchwabNraTax {
    pub(crate) date: NaiveDate,
    pub(crate) symbol: Option<String>,
    pub(crate) amount: Option<Decimal>,
}

#[derive(Debug, Clone)]
pub(crate) enum SchwabTransaction {
    Buy(SchwabTrade),
    Sell(SchwabTrade),
    CancelSell(SchwabTrade),
    StockPlanActivity(SchwabStockPlanActivity),
    CashDividend(SchwabDividend),
    QualifiedDividend(SchwabDividend),
    ShortTermCapGain(SchwabDividend),
    LongTermCapGain(SchwabDividend),
    StockSplit(SchwabStockSplit),
    NraTaxAdj(SchwabNraTax),
    NraWithholding(SchwabNraTax),
    /// Known action that is not relevant for CGT (cash movements, fees, etc.)
    NonCgt,
}

#[derive(Debug, Clone)]
pub(crate) struct CommonFields {
    pub(crate) date: NaiveDate,
    pub(crate) symbol: String,
}

#[derive(Debug, Clone)]
pub(crate) struct SchwabRawTransaction {
    date: Option<String>,
    action: Option<String>,
    symbol: Option<String>,
    description: Option<String>,
}

pub(crate) fn parse_transactions_json(
    json_content: &str,
) -> Result<Vec<SchwabTransactionsItem>, ConvertError> {
    let payload: SchwabTransactions = serde_json::from_str(json_content)?;
    payload
        .brokerage_transactions
        .into_iter()
        .map(parse_transaction_value)
        .collect()
}

pub(crate) fn format_unknown_comment(raw: &SchwabRawTransaction) -> String {
    let action = raw.action.as_deref().unwrap_or("Unknown");
    let symbol = raw.symbol.as_deref().unwrap_or("");
    let description = raw.description.as_deref().unwrap_or("");
    let raw_date = raw.date.as_deref().unwrap_or("");
    let formatted_date = raw
        .date
        .as_deref()
        .and_then(|date| parse_date(date).ok())
        .map(|date| date.format("%Y-%m-%d").to_string())
        .unwrap_or_else(|| raw_date.to_string());

    format!(
        "SKIPPED: {} - {} on {} ({})",
        action, symbol, formatted_date, description
    )
}

fn parse_transaction_value(value: Value) -> Result<SchwabTransactionsItem, ConvertError> {
    let action = value
        .get(KEY_ACTION)
        .and_then(|field| field.as_str())
        .map(str::trim)
        .filter(|text| !text.is_empty());

    let Some(action) = action else {
        return Err(ConvertError::InvalidTransaction(
            "Missing Action field".to_string(),
        ));
    };

    if let Some(action) = SchwabAction::parse(action) {
        let txn = parse_known_transaction(action, &value)?;
        Ok(SchwabTransactionsItem::Known(txn))
    } else {
        Ok(SchwabTransactionsItem::Unknown(parse_raw_transaction(
            &value,
        )))
    }
}

fn parse_known_transaction(
    action: SchwabAction,
    value: &Value,
) -> Result<SchwabTransaction, ConvertError> {
    match action {
        SchwabAction::Buy => Ok(SchwabTransaction::Buy(parse_trade(action, value)?)),
        SchwabAction::Sell => Ok(SchwabTransaction::Sell(parse_trade(action, value)?)),
        SchwabAction::CancelSell => Ok(SchwabTransaction::CancelSell(parse_trade(action, value)?)),
        SchwabAction::StockPlanActivity => Ok(SchwabTransaction::StockPlanActivity(
            parse_stock_plan_activity(value)?,
        )),
        SchwabAction::CashDividend => Ok(SchwabTransaction::CashDividend(parse_dividend(value)?)),
        SchwabAction::QualifiedDividend => {
            Ok(SchwabTransaction::QualifiedDividend(parse_dividend(value)?))
        }
        SchwabAction::ShortTermCapGain => {
            Ok(SchwabTransaction::ShortTermCapGain(parse_dividend(value)?))
        }
        SchwabAction::LongTermCapGain => {
            Ok(SchwabTransaction::LongTermCapGain(parse_dividend(value)?))
        }
        SchwabAction::StockSplit => Ok(SchwabTransaction::StockSplit(parse_stock_split(value)?)),
        SchwabAction::NraTaxAdj => Ok(SchwabTransaction::NraTaxAdj(parse_nra_tax(value)?)),
        SchwabAction::NraWithholding => {
            Ok(SchwabTransaction::NraWithholding(parse_nra_tax(value)?))
        }
        SchwabAction::Adjustment
        | SchwabAction::CreditInterest
        | SchwabAction::Journal
        | SchwabAction::MiscCashEntry
        | SchwabAction::MoneyLinkTransfer
        | SchwabAction::ServiceFee
        | SchwabAction::WireFundsAdj
        | SchwabAction::WireSent => Ok(SchwabTransaction::NonCgt),
    }
}

fn parse_trade(action: SchwabAction, value: &Value) -> Result<SchwabTrade, ConvertError> {
    let common = parse_common_fields(value)?;
    let quantity = parse_required_decimal_field(action, value, KEY_QUANTITY, LABEL_QUANTITY)?;
    let price = parse_required_decimal_field(action, value, KEY_PRICE, LABEL_PRICE)?;
    let fees_commissions = parse_optional_decimal_field(value, KEY_FEES)?;

    Ok(SchwabTrade {
        common,
        quantity,
        price,
        fees_commissions,
    })
}

fn parse_stock_plan_activity(value: &Value) -> Result<SchwabStockPlanActivity, ConvertError> {
    let common = parse_common_fields(value)?;
    let quantity = parse_required_decimal_field(
        SchwabAction::StockPlanActivity,
        value,
        KEY_QUANTITY,
        LABEL_QUANTITY,
    )?;

    Ok(SchwabStockPlanActivity { common, quantity })
}

fn parse_dividend(value: &Value) -> Result<SchwabDividend, ConvertError> {
    let common = parse_common_fields(value)?;
    let amount = parse_optional_decimal_field(value, KEY_AMOUNT)?;

    Ok(SchwabDividend { common, amount })
}

fn parse_stock_split(value: &Value) -> Result<SchwabStockSplit, ConvertError> {
    let common = parse_common_fields(value)?;
    Ok(SchwabStockSplit { common })
}

fn parse_nra_tax(value: &Value) -> Result<SchwabNraTax, ConvertError> {
    let date_raw = get_required_string(value, KEY_DATE, KEY_DATE)?;
    let date = parse_date(date_raw)?;
    let symbol = get_optional_string(value, KEY_SYMBOL).map(str::to_string);
    let amount = parse_optional_decimal_field(value, KEY_AMOUNT)?;

    Ok(SchwabNraTax {
        date,
        symbol,
        amount,
    })
}

fn parse_common_fields(value: &Value) -> Result<CommonFields, ConvertError> {
    let date_raw = get_required_string(value, KEY_DATE, KEY_DATE)?;
    let symbol = get_required_string(value, KEY_SYMBOL, KEY_SYMBOL)?.to_string();
    let date = parse_date(date_raw)?;

    Ok(CommonFields { date, symbol })
}

fn parse_raw_transaction(value: &Value) -> SchwabRawTransaction {
    SchwabRawTransaction {
        date: get_optional_string(value, KEY_DATE).map(str::to_string),
        action: get_optional_string(value, KEY_ACTION).map(str::to_string),
        symbol: get_optional_string(value, KEY_SYMBOL).map(str::to_string),
        description: get_optional_string(value, KEY_DESCRIPTION).map(str::to_string),
    }
}

fn get_required_string<'a>(
    value: &'a Value,
    key: &'static str,
    label: &str,
) -> Result<&'a str, ConvertError> {
    let raw = value
        .get(key)
        .and_then(|field| field.as_str())
        .map(str::trim)
        .filter(|text| !text.is_empty())
        .ok_or_else(|| ConvertError::InvalidTransaction(format!("Missing {}", label)))?;

    Ok(raw)
}

fn get_optional_string<'a>(value: &'a Value, key: &'static str) -> Option<&'a str> {
    value
        .get(key)
        .and_then(|field| field.as_str())
        .map(str::trim)
        .filter(|text| !text.is_empty())
}

fn parse_required_decimal_field(
    action: SchwabAction,
    value: &Value,
    key: &'static str,
    label: &str,
) -> Result<Decimal, ConvertError> {
    let raw = get_required_string(value, key, label)?;
    parse_amount(raw)?.ok_or_else(|| {
        ConvertError::InvalidTransaction(format!("{} missing {}", action.as_str(), label))
    })
}

fn parse_optional_decimal_field(
    value: &Value,
    key: &'static str,
) -> Result<Option<Decimal>, ConvertError> {
    match get_optional_string(value, key) {
        Some(amount) => parse_amount(amount),
        None => Ok(None),
    }
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
