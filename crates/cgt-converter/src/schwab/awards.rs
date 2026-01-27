use crate::error::ConvertError;
use chrono::NaiveDate;
use rust_decimal::Decimal;
use serde::Deserialize;
use std::collections::HashMap;

/// Equity awards data parsed from Schwab exports
#[derive(Debug, Clone)]
pub struct AwardsData {
    /// Map of (symbol, date) -> Fair Market Value
    fmv_map: HashMap<(String, NaiveDate), Decimal>,
}

/// Result of looking up an award by date
#[derive(Debug, Clone)]
pub struct AwardLookup {
    /// The Fair Market Value at vest
    pub fmv: Decimal,
    /// The actual vest/lapse date (for CGT acquisition date)
    pub vest_date: NaiveDate,
}

impl AwardsData {
    /// Get Fair Market Value and vest date for a specific symbol and date
    /// Implements 7-day lookback for date matching
    ///
    /// Returns the FMV and the actual vest date found (which may differ from
    /// the query date due to T+2 settlement). The vest date should be used
    /// as the CGT acquisition date per HMRC guidance.
    pub fn get_fmv(&self, date: &NaiveDate, symbol: &str) -> Result<AwardLookup, ConvertError> {
        // Normalize symbol to uppercase for case-insensitive lookup
        let symbol_upper = symbol.to_uppercase();

        // Try exact match first
        if let Some(fmv) = self.fmv_map.get(&(symbol_upper.clone(), *date)) {
            return Ok(AwardLookup {
                fmv: *fmv,
                vest_date: *date,
            });
        }

        // Try 7-day lookback (for awards that may have slightly different dates)
        for days_back in 1..=7 {
            if let Some(earlier_date) = date.checked_sub_signed(chrono::Duration::days(days_back))
                && let Some(fmv) = self.fmv_map.get(&(symbol_upper.clone(), earlier_date))
            {
                return Ok(AwardLookup {
                    fmv: *fmv,
                    vest_date: earlier_date,
                });
            }
        }

        Err(ConvertError::MissingFairMarketValue {
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
    #[serde(rename = "TransactionDetails")]
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

fn parse_optional_price(price_str: &str) -> Result<Option<Decimal>, ConvertError> {
    let trimmed = price_str.trim();
    if trimmed.is_empty() || trimmed == "--" {
        return Ok(None);
    }

    let cleaned = trimmed.replace(['$', ','], "");
    cleaned
        .parse::<Decimal>()
        .map(Some)
        .map_err(|_| ConvertError::InvalidAmount(price_str.to_string()))
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
        let fmv = parse_optional_price(vest_fmv_str)?;
        return Ok((Some(vest_date), fmv, true));
    }

    if let Some(fmv_str) = details.fair_market_value_price.as_deref() {
        let fmv = parse_optional_price(fmv_str)?;
        return Ok((Some(parent_date), fmv, false));
    }

    Ok((None, None, false))
}

/// Parse Schwab equity awards JSON
pub fn parse_awards_json(json_content: &str) -> Result<AwardsData, ConvertError> {
    let awards_json: AwardsJson = serde_json::from_str(json_content)?;

    let mut fmv_map = HashMap::new();

    for award in awards_json.transactions {
        let parent_date = parse_award_date(&award.date)?;
        let action_kind = classify_award_action(award.action.as_deref());

        if award.transaction_details.is_empty() {
            if action_kind == AwardAction::NonVesting {
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
        let symbol_upper = award.symbol.to_uppercase();
        let mut fallback: Option<(NaiveDate, Decimal)> = None;
        let mut inserted = false;

        for detail in &award.transaction_details {
            let (date, fmv, is_vest) = extract_award_fmv(&detail.details, parent_date)?;
            if let (Some(date), Some(fmv)) = (date, fmv) {
                if is_vest {
                    // Insert all vest FMVs found (multiple grants may vest on different dates)
                    fmv_map.insert((symbol_upper.clone(), date), fmv);
                    inserted = true;
                }

                if fallback.is_none() {
                    fallback = Some((date, fmv));
                }
            }
        }

        if !inserted && let Some((date, fmv)) = fallback {
            fmv_map.insert((symbol_upper, date), fmv);
        }
    }

    Ok(AwardsData { fmv_map })
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
    fn test_unknown_action_missing_transaction_details_fails() {
        let json = r#"{"Transactions": [
            {"Date": "04/25/2023", "Action": "Mystery Action", "Symbol": "XYZZ", "TransactionDetails": []}
        ]}"#;

        let result = parse_awards_json(json);
        assert!(matches!(result, Err(ConvertError::InvalidTransaction(_))));
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
