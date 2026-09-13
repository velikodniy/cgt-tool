use crate::error::ConvertError;
use chrono::NaiveDate;
use rust_decimal::Decimal;
use serde::Deserialize;
use std::collections::HashMap;

/// Equity awards retain every candidate until the deposit is resolved.
#[derive(Debug, Clone)]
pub struct AwardsData {
    by_symbol: HashMap<String, Vec<Award>>,
}

/// Fair market value and acquisition date from an award.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AwardLookup {
    pub fmv: Decimal,
    pub vest_date: NaiveDate,
}

#[derive(Debug, Clone)]
struct Award {
    value: AwardLookup,
    deposit: Option<Deposit>,
}

#[derive(Debug, Clone, Copy)]
struct Deposit {
    date: NaiveDate,
    quantity: Option<Decimal>,
}

impl AwardsData {
    fn candidates(&self, symbol: &str) -> &[Award] {
        self.by_symbol
            .get(&symbol.to_uppercase())
            .map(Vec::as_slice)
            .unwrap_or_default()
    }

    pub(crate) fn get_fmv_for_deposit(
        &self,
        date: &NaiveDate,
        symbol: &str,
        quantity: Decimal,
    ) -> Result<AwardLookup, ConvertError> {
        let awards = self.candidates(symbol);
        let has_deposit = awards
            .iter()
            .any(|award| award.deposit.is_some_and(|deposit| deposit.date == *date));
        let mut candidates = awards.iter().filter(|award| {
            if has_deposit {
                award.deposit.is_some_and(|deposit| {
                    deposit.date == *date
                        && deposit.quantity.is_none_or(|amount| amount == quantity)
                })
            } else {
                (0..=7).contains(&date.signed_duration_since(award.value.vest_date).num_days())
            }
        });
        let first = candidates
            .next()
            .ok_or_else(|| ConvertError::MissingFairMarketValue {
                date: date.to_string(),
                symbol: symbol.to_string(),
            })?;
        if candidates.any(|candidate| candidate.value != first.value) {
            return Err(ConvertError::InvalidTransaction(format!(
                "Ambiguous awards for {symbol} deposit of {quantity} shares on {date}"
            )));
        }
        Ok(first.value.clone())
    }

    /// Get the closest vest within seven days of the supplied date.
    /// When entries share a vest date, the last export entry wins.
    /// Conversion uses deposit reconciliation instead of this legacy lookup.
    pub fn get_fmv(&self, date: &NaiveDate, symbol: &str) -> Result<AwardLookup, ConvertError> {
        self.candidates(symbol)
            .iter()
            .rev()
            .filter(|award| {
                (0..=7).contains(&date.signed_duration_since(award.value.vest_date).num_days())
            })
            .min_by_key(|award| date.signed_duration_since(award.value.vest_date).num_days())
            .map(|award| award.value.clone())
            .ok_or_else(|| ConvertError::MissingFairMarketValue {
                date: date.to_string(),
                symbol: symbol.to_string(),
            })
    }
}

/// Schwab equity awards JSON structure
#[derive(Debug, Deserialize)]
struct AwardsJson {
    #[serde(rename = "Transactions")]
    transactions: Vec<AwardTransaction>,
}

#[derive(Debug, Deserialize)]
struct AwardTransaction {
    #[serde(rename = "Date")]
    date: String,
    #[serde(rename = "Action")]
    action: Option<String>,
    #[serde(rename = "Symbol")]
    symbol: String,
    #[serde(rename = "Quantity")]
    quantity: Option<String>,
    #[serde(rename = "TransactionDetails")]
    #[serde(default)]
    transaction_details: Vec<AwardTransactionDetails>,
}

#[derive(Debug, Deserialize)]
struct AwardTransactionDetails {
    #[serde(rename = "Details")]
    details: AwardDetails,
}

#[derive(Debug, Deserialize)]
struct AwardDetails {
    #[serde(rename = "FairMarketValuePrice")]
    fair_market_value_price: Option<String>,
    #[serde(rename = "VestDate")]
    vest_date: Option<String>,
    #[serde(rename = "VestFairMarketValue")]
    vest_fair_market_value: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum AwardAction {
    Vesting,
    NonVesting,
    Unknown,
}

fn classify_award_action(action: Option<&str>) -> AwardAction {
    match action.map(str::trim) {
        Some("Deposit") | Some("Lapse") | Some("Sale") | Some("Forced Quick Sell") => {
            AwardAction::Vesting
        }
        Some("Wire Transfer")
        | Some("Tax Withholding")
        | Some("Tax Reversal")
        | Some("Forced Disbursement") => AwardAction::NonVesting,
        Some(_) | None => AwardAction::Unknown,
    }
}

fn parse_award_date(date_str: &str) -> Result<NaiveDate, ConvertError> {
    NaiveDate::parse_from_str(date_str, "%m/%d/%Y")
        .map_err(|_| ConvertError::InvalidDate(date_str.to_string()))
}

fn extract_award_fmv(
    details: &AwardDetails,
    parent_date: NaiveDate,
) -> Result<(Option<NaiveDate>, Option<Decimal>, bool), ConvertError> {
    if let Some(vest_fmv_str) = details.vest_fair_market_value.as_deref() {
        let vest_date = details
            .vest_date
            .as_deref()
            .map(parse_award_date)
            .transpose()?
            .unwrap_or(parent_date);
        let fmv = super::parse_dollar_amount(vest_fmv_str)?;
        return Ok((Some(vest_date), fmv, true));
    }

    if let Some(fmv_str) = details.fair_market_value_price.as_deref() {
        let fmv = super::parse_dollar_amount(fmv_str)?;
        return Ok((Some(parent_date), fmv, false));
    }

    Ok((None, None, false))
}

/// Parse Schwab equity awards JSON
pub fn parse_awards_json(json_content: &str) -> Result<AwardsData, ConvertError> {
    let awards_json: AwardsJson = serde_json::from_str(json_content)?;

    let mut by_symbol: HashMap<String, Vec<Award>> = HashMap::new();

    for award in awards_json.transactions {
        let parent_date = parse_award_date(&award.date)?;
        let action_kind = classify_award_action(award.action.as_deref());

        if award.transaction_details.is_empty() {
            if action_kind != AwardAction::Vesting {
                continue;
            }

            let action_label = award
                .action
                .as_deref()
                .map(str::trim)
                .filter(|value| !value.is_empty())
                .unwrap_or("Unknown");
            return Err(ConvertError::InvalidTransaction(format!(
                "Award entry missing TransactionDetails for {} on {} (action: {})",
                award.symbol, award.date, action_label
            )));
        }
        let symbol_upper = award.symbol.trim().to_uppercase();
        let is_deposit = matches!(
            award.action.as_deref().map(str::trim),
            Some("Deposit") | None
        );
        let quantity = award
            .quantity
            .as_deref()
            .map(super::parse_dollar_amount)
            .transpose()?
            .flatten();
        let deposit = is_deposit.then_some(Deposit {
            date: parent_date,
            quantity,
        });
        let mut vest_values = Vec::new();
        let mut fallback_values = Vec::new();
        for detail in &award.transaction_details {
            let (date, fmv, is_vest) = extract_award_fmv(&detail.details, parent_date)?;
            if let (Some(vest_date), Some(fmv)) = (date, fmv) {
                let record = Award {
                    value: AwardLookup { fmv, vest_date },
                    deposit,
                };
                if is_vest {
                    vest_values.push(record);
                } else {
                    fallback_values.push(record);
                }
            }
        }
        let values = if vest_values.is_empty() {
            fallback_values
        } else {
            vest_values
        };
        by_symbol.entry(symbol_upper).or_default().extend(values);
    }

    Ok(AwardsData { by_symbol })
}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal_macros::dec;

    #[test]
    fn test_parse_awards_json() {
        let json = r#"{
            "Transactions": [
                {
                    "Date": "04/25/2023",
                    "Symbol": "XYZZ",
                    "TransactionDetails": [
                        {"Details": {"FairMarketValuePrice": "$125.6445"}}
                    ]
                },
                {
                    "Date": "06/15/2023",
                    "Symbol": "BAR",
                    "TransactionDetails": [
                        {"Details": {"FairMarketValuePrice": "340.50"}}
                    ]
                }
            ]
        }"#;

        let awards = parse_awards_json(json).unwrap();
        let date1 = NaiveDate::from_ymd_opt(2023, 4, 25).unwrap();
        let date2 = NaiveDate::from_ymd_opt(2023, 6, 15).unwrap();

        let lookup1 = awards.get_fmv(&date1, "XYZZ").unwrap();
        assert_eq!(lookup1.fmv, dec!(125.6445));
        assert_eq!(lookup1.vest_date, date1);

        let lookup2 = awards.get_fmv(&date2, "BAR").unwrap();
        assert_eq!(lookup2.fmv, dec!(340.50));
        assert_eq!(lookup2.vest_date, date2);
    }

    #[test]
    fn test_missing_fmv() {
        let json = r#"{"Transactions": []}"#;
        let awards = parse_awards_json(json).unwrap();

        let date = NaiveDate::from_ymd_opt(2023, 4, 25).unwrap();
        let result = awards.get_fmv(&date, "XYZZ");

        let Err(ConvertError::MissingFairMarketValue { date, symbol }) = result else {
            unreachable!("Expected MissingFairMarketValue error");
        };
        assert_eq!(symbol, "XYZZ");
        assert!(date.contains("2023-04-25"));
    }

    #[test]
    fn test_json_duplicate_entries_last_wins() {
        let json = r#"{"Transactions": [
            {"Date": "04/25/2023", "Symbol": "XYZZ", "TransactionDetails": [{"Details": {"FairMarketValuePrice": "100.00"}}]},
            {"Date": "04/25/2023", "Symbol": "XYZZ", "TransactionDetails": [{"Details": {"FairMarketValuePrice": "125.00"}}]}
        ]}"#;

        let awards = parse_awards_json(json).unwrap();
        let date = NaiveDate::from_ymd_opt(2023, 4, 25).unwrap();
        assert_eq!(awards.get_fmv(&date, "XYZZ").unwrap().fmv, dec!(125.00));
    }

    #[test]
    fn test_wire_transfer_with_empty_details_accepted() {
        let json = r#"{"Transactions": [
            {"Date": "04/25/2023", "Action": "Wire Transfer", "Symbol": "XYZZ", "TransactionDetails": []}
        ]}"#;

        let awards = parse_awards_json(json).unwrap();
        let date = NaiveDate::from_ymd_opt(2023, 4, 25).unwrap();
        let result = awards.get_fmv(&date, "XYZZ");

        assert!(matches!(
            result,
            Err(ConvertError::MissingFairMarketValue { .. })
        ));
    }

    #[test]
    fn test_unknown_action_missing_transaction_details_is_ignored() {
        let json = r#"{"Transactions": [
            {"Date": "04/25/2023", "Action": "Mystery Action", "Symbol": "XYZZ", "TransactionDetails": []}
        ]}"#;

        let awards = parse_awards_json(json).unwrap();
        let date = NaiveDate::from_ymd_opt(2023, 4, 25).unwrap();
        let result = awards.get_fmv(&date, "XYZZ");

        assert!(matches!(
            result,
            Err(ConvertError::MissingFairMarketValue { .. })
        ));
    }

    #[test]
    fn test_json_fmv_various_formats() {
        let json = r#"{"Transactions": [
            {"Date": "04/25/2023", "Symbol": "A", "TransactionDetails": [{"Details": {"FairMarketValuePrice": "$125.00"}}]},
            {"Date": "04/25/2023", "Symbol": "B", "TransactionDetails": [{"Details": {"FairMarketValuePrice": "125.00"}}]},
            {"Date": "04/25/2023", "Symbol": "C", "TransactionDetails": [{"Details": {"FairMarketValuePrice": "$125.6445"}}]},
            {"Date": "04/25/2023", "Symbol": "D", "TransactionDetails": [{"Details": {"FairMarketValuePrice": "0.50"}}]}
        ]}"#;

        let awards = parse_awards_json(json).unwrap();
        let date = NaiveDate::from_ymd_opt(2023, 4, 25).unwrap();

        assert_eq!(awards.get_fmv(&date, "A").unwrap().fmv, dec!(125.00));
        assert_eq!(awards.get_fmv(&date, "B").unwrap().fmv, dec!(125.00));
        assert_eq!(awards.get_fmv(&date, "C").unwrap().fmv, dec!(125.6445));
        assert_eq!(awards.get_fmv(&date, "D").unwrap().fmv, dec!(0.50));
    }

    #[test]
    fn test_parse_awards_json_with_vest_fields() {
        let json = r#"{
            "Transactions": [
                {
                    "Date": "04/28/2023",
                    "Symbol": "XYZZ",
                    "TransactionDetails": [
                        {"Details": {"VestDate": "04/25/2023", "VestFairMarketValue": "$125.50"}}
                    ]
                }
            ]
        }"#;

        let awards = parse_awards_json(json).unwrap();
        let vest_date = NaiveDate::from_ymd_opt(2023, 4, 25).unwrap();

        // Query directly by vest date (not settlement date) to verify vest field extraction
        let lookup = awards.get_fmv(&vest_date, "XYZZ").unwrap();
        assert_eq!(lookup.fmv, dec!(125.50));
        assert_eq!(lookup.vest_date, vest_date);
    }

    #[test]
    fn test_vest_fields_stored_by_vest_date_not_parent_date() {
        let json = r#"{
            "Transactions": [
                {
                    "Date": "04/28/2023",
                    "Symbol": "XYZZ",
                    "TransactionDetails": [
                        {"Details": {"VestDate": "04/25/2023", "VestFairMarketValue": "$125.50"}}
                    ]
                }
            ]
        }"#;

        let awards = parse_awards_json(json).unwrap();
        let vest_date = NaiveDate::from_ymd_opt(2023, 4, 25).unwrap();

        // FMV is stored under vest date, not parent date (04/28)
        let lookup = awards.get_fmv(&vest_date, "XYZZ").unwrap();
        assert_eq!(lookup.fmv, dec!(125.50));

        // A query by the parent date (04/28) can still resolve via the 7-day lookback.
        // Use a date outside the lookback range to verify vest-date keyed storage.
        let outside_lookback = NaiveDate::from_ymd_opt(2023, 5, 5).unwrap();
        assert!(awards.get_fmv(&outside_lookback, "XYZZ").is_err());

        // But within lookback from vest date should work
        let within_lookback = NaiveDate::from_ymd_opt(2023, 5, 1).unwrap();
        let lookup2 = awards.get_fmv(&within_lookback, "XYZZ").unwrap();
        assert_eq!(lookup2.vest_date, vest_date);
    }

    #[test]
    fn test_missing_fmv_in_details_returns_missing_error() {
        let json = r#"{
            "Transactions": [
                {
                    "Date": "04/25/2023",
                    "Symbol": "XYZZ",
                    "TransactionDetails": [
                        {"Details": {}}
                    ]
                }
            ]
        }"#;

        let awards = parse_awards_json(json).unwrap();
        let date = NaiveDate::from_ymd_opt(2023, 4, 25).unwrap();
        let result = awards.get_fmv(&date, "XYZZ");

        assert!(matches!(
            result,
            Err(ConvertError::MissingFairMarketValue { .. })
        ));
    }

    #[test]
    fn test_multiple_vest_entries_all_inserted() {
        // Multiple TransactionDetails with different vest dates should all be inserted
        let json = r#"{
            "Transactions": [
                {
                    "Date": "04/28/2023",
                    "Symbol": "XYZZ",
                    "TransactionDetails": [
                        {"Details": {"VestDate": "04/20/2023", "VestFairMarketValue": "$100.00"}},
                        {"Details": {"VestDate": "04/25/2023", "VestFairMarketValue": "$125.50"}}
                    ]
                }
            ]
        }"#;

        let awards = parse_awards_json(json).unwrap();
        let date1 = NaiveDate::from_ymd_opt(2023, 4, 20).unwrap();
        let date2 = NaiveDate::from_ymd_opt(2023, 4, 25).unwrap();

        // Both vest entries should be accessible
        let lookup1 = awards.get_fmv(&date1, "XYZZ").unwrap();
        assert_eq!(lookup1.fmv, dec!(100.00));
        assert_eq!(lookup1.vest_date, date1);

        let lookup2 = awards.get_fmv(&date2, "XYZZ").unwrap();
        assert_eq!(lookup2.fmv, dec!(125.50));
        assert_eq!(lookup2.vest_date, date2);
    }

    #[test]
    fn test_fmv_lookback_boundary_7_days() {
        // Test exact boundary of 7-day lookback
        let json = r#"{
            "Transactions": [
                {
                    "Date": "04/20/2023",
                    "Symbol": "XYZZ",
                    "TransactionDetails": [
                        {"Details": {"FairMarketValuePrice": "125.00"}}
                    ]
                }
            ]
        }"#;
        let awards = parse_awards_json(json).unwrap();
        let vest_date = NaiveDate::from_ymd_opt(2023, 4, 20).unwrap();

        let day0 = NaiveDate::from_ymd_opt(2023, 4, 20).unwrap();
        let lookup = awards.get_fmv(&day0, "XYZZ").unwrap();
        assert_eq!(lookup.fmv, dec!(125.00));
        assert_eq!(lookup.vest_date, vest_date);

        let day7 = NaiveDate::from_ymd_opt(2023, 4, 27).unwrap();
        let lookup = awards.get_fmv(&day7, "XYZZ").unwrap();
        assert_eq!(lookup.fmv, dec!(125.00));
        assert_eq!(lookup.vest_date, vest_date);

        let day8 = NaiveDate::from_ymd_opt(2023, 4, 28).unwrap();
        assert!(awards.get_fmv(&day8, "XYZZ").is_err());
    }

    #[test]
    fn test_fmv_lookback_prefers_exact_match() {
        let json = r#"{
            "Transactions": [
                {
                    "Date": "04/20/2023",
                    "Symbol": "XYZZ",
                    "TransactionDetails": [
                        {"Details": {"FairMarketValuePrice": "120.00"}}
                    ]
                },
                {
                    "Date": "04/25/2023",
                    "Symbol": "XYZZ",
                    "TransactionDetails": [
                        {"Details": {"FairMarketValuePrice": "125.00"}}
                    ]
                }
            ]
        }"#;
        let awards = parse_awards_json(json).unwrap();

        let exact_date = NaiveDate::from_ymd_opt(2023, 4, 25).unwrap();
        let lookup = awards.get_fmv(&exact_date, "XYZZ").unwrap();
        assert_eq!(lookup.fmv, dec!(125.00));
        assert_eq!(lookup.vest_date, exact_date);
    }

    #[test]
    fn test_fmv_lookback_finds_closest() {
        let json = r#"{
            "Transactions": [
                {
                    "Date": "04/20/2023",
                    "Symbol": "XYZZ",
                    "TransactionDetails": [
                        {"Details": {"FairMarketValuePrice": "120.00"}}
                    ]
                },
                {
                    "Date": "04/23/2023",
                    "Symbol": "XYZZ",
                    "TransactionDetails": [
                        {"Details": {"FairMarketValuePrice": "123.00"}}
                    ]
                }
            ]
        }"#;
        let awards = parse_awards_json(json).unwrap();

        let query_date = NaiveDate::from_ymd_opt(2023, 4, 25).unwrap();
        let expected_vest_date = NaiveDate::from_ymd_opt(2023, 4, 23).unwrap();
        let lookup = awards.get_fmv(&query_date, "XYZZ").unwrap();
        assert_eq!(lookup.fmv, dec!(123.00));
        assert_eq!(lookup.vest_date, expected_vest_date);
    }

    #[test]
    fn test_fmv_case_insensitive_symbol() {
        let json = r#"{
            "Transactions": [
                {
                    "Date": "04/25/2023",
                    "Symbol": "XYZZ",
                    "TransactionDetails": [
                        {"Details": {"FairMarketValuePrice": "125.00"}}
                    ]
                }
            ]
        }"#;
        let awards = parse_awards_json(json).unwrap();

        let date = NaiveDate::from_ymd_opt(2023, 4, 25).unwrap();
        assert_eq!(awards.get_fmv(&date, "XYZZ").unwrap().fmv, dec!(125.00));
        assert_eq!(awards.get_fmv(&date, "xyzz").unwrap().fmv, dec!(125.00));
        assert_eq!(awards.get_fmv(&date, "Xyzz").unwrap().fmv, dec!(125.00));
        assert!(awards.get_fmv(&date, "UNKNOWN").is_err());
    }

    #[test]
    fn test_fmv_cross_month_lookback() {
        let json = r#"{
            "Transactions": [
                {
                    "Date": "03/28/2023",
                    "Symbol": "XYZZ",
                    "TransactionDetails": [
                        {"Details": {"FairMarketValuePrice": "125.00"}}
                    ]
                }
            ]
        }"#;
        let awards = parse_awards_json(json).unwrap();

        let vest_date = NaiveDate::from_ymd_opt(2023, 3, 28).unwrap();
        let april_date = NaiveDate::from_ymd_opt(2023, 4, 3).unwrap();
        let lookup = awards.get_fmv(&april_date, "XYZZ").unwrap();
        assert_eq!(lookup.fmv, dec!(125.00));
        assert_eq!(lookup.vest_date, vest_date);
    }

    #[test]
    fn test_fmv_cross_year_lookback() {
        let json = r#"{
            "Transactions": [
                {
                    "Date": "12/28/2022",
                    "Symbol": "XYZZ",
                    "TransactionDetails": [
                        {"Details": {"FairMarketValuePrice": "125.00"}}
                    ]
                }
            ]
        }"#;
        let awards = parse_awards_json(json).unwrap();

        let vest_date = NaiveDate::from_ymd_opt(2022, 12, 28).unwrap();
        let jan_date = NaiveDate::from_ymd_opt(2023, 1, 3).unwrap();
        let lookup = awards.get_fmv(&jan_date, "XYZZ").unwrap();
        assert_eq!(lookup.fmv, dec!(125.00));
        assert_eq!(lookup.vest_date, vest_date);
    }
}
