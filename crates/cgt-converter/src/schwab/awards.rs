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
    fair_market_value_price: String,
}

/// Parse Schwab equity awards JSON
pub fn parse_awards_json(json_content: &str) -> Result<AwardsData, ConvertError> {
    let awards_json: AwardsJson = serde_json::from_str(json_content)?;

    let mut fmv_map = HashMap::new();

    for award in awards_json.transactions {
        let date = NaiveDate::parse_from_str(&award.date, "%m/%d/%Y")
            .map_err(|_| ConvertError::InvalidDate(award.date.clone()))?;

        let details = award.transaction_details.first().ok_or_else(|| {
            ConvertError::InvalidTransaction(format!(
                "Award entry missing TransactionDetails for {} on {}",
                award.symbol, award.date
            ))
        })?;

        let price_str = details
            .details
            .fair_market_value_price
            .trim()
            .replace(['$', ','], "");
        let price = price_str.parse::<Decimal>().map_err(|_| {
            ConvertError::InvalidAmount(details.details.fair_market_value_price.clone())
        })?;

        fmv_map.insert((award.symbol.to_uppercase(), date), price);
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
    fn test_json_missing_transaction_details() {
        let json = r#"{"Transactions": [
            {"Date": "04/25/2023", "Symbol": "XYZZ", "TransactionDetails": []}
        ]}"#;

        let result = parse_awards_json(json);
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            ConvertError::InvalidTransaction(_)
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
