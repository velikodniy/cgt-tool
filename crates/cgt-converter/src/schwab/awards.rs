use crate::error::ConvertError;
use chrono::NaiveDate;
use csv::StringRecord;
use rust_decimal::Decimal;
use serde::Deserialize;
use std::collections::HashMap;

/// Awards file format
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AwardsFormat {
    /// JSON format
    Json,
    /// CSV format
    Csv,
}

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
        // Try exact match first
        if let Some(fmv) = self.fmv_map.get(&(symbol.to_string(), *date)) {
            return Ok(AwardLookup {
                fmv: *fmv,
                vest_date: *date,
            });
        }

        // Try 7-day lookback (for awards that may have slightly different dates)
        for days_back in 1..=7 {
            if let Some(earlier_date) = date.checked_sub_signed(chrono::Duration::days(days_back))
                && let Some(fmv) = self.fmv_map.get(&(symbol.to_string(), earlier_date))
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
    #[serde(rename = "EquityAwards")]
    equity_awards: Vec<Award>,
}

#[derive(Debug, Deserialize)]
struct Award {
    #[serde(rename = "Symbol")]
    symbol: String,
    #[serde(rename = "EventDate")]
    event_date: String,
    #[serde(rename = "FairMarketValuePrice")]
    fair_market_value_price: String,
}

/// Parse Schwab equity awards with specified format
pub fn parse_awards(content: &str, format: AwardsFormat) -> Result<AwardsData, ConvertError> {
    match format {
        AwardsFormat::Json => parse_awards_json(content),
        AwardsFormat::Csv => parse_awards_csv(content),
    }
}

/// Parse Schwab equity awards JSON
fn parse_awards_json(json_content: &str) -> Result<AwardsData, ConvertError> {
    let awards_json: AwardsJson = serde_json::from_str(json_content)?;

    let mut fmv_map = HashMap::new();

    for award in awards_json.equity_awards {
        // Parse date (MM/DD/YYYY format in Schwab JSON)
        let date = NaiveDate::parse_from_str(&award.event_date, "%m/%d/%Y")
            .map_err(|_| ConvertError::InvalidDate(award.event_date.clone()))?;

        // Parse FMV price (may have $ prefix)
        let price_str = award.fair_market_value_price.trim().replace('$', "");
        let price = price_str
            .parse::<Decimal>()
            .map_err(|_| ConvertError::InvalidAmount(award.fair_market_value_price.clone()))?;

        fmv_map.insert((award.symbol, date), price);
    }

    Ok(AwardsData { fmv_map })
}

/// Parse Schwab equity awards CSV
///
/// Schwab CSV format uses paired rows:
/// - Row 1: Transaction details (Date=lapse/vest date, Action="Lapse", Symbol, etc.)
/// - Row 2: Award details (AwardDate=original grant date, FairMarketValuePrice)
///
/// The FMV is looked up by (symbol, lapse_date) - the Date from the transaction row,
/// NOT the AwardDate (which is the original grant date).
fn parse_awards_csv(csv_content: &str) -> Result<AwardsData, ConvertError> {
    let mut reader = csv::ReaderBuilder::new()
        .has_headers(true)
        .flexible(true) // Allow variable number of fields per record
        .from_reader(csv_content.as_bytes());

    let headers = reader.headers()?.clone();

    // Validate required columns exist
    let date_idx = find_column_index(&headers, "Date")?;
    let symbol_idx = find_column_index(&headers, "Symbol")?;
    let fmv_idx = find_column_index(&headers, "FairMarketValuePrice")?;

    let mut fmv_map = HashMap::new();
    let mut current_lapse: Option<(String, NaiveDate)> = None; // (symbol, lapse_date)

    for result in reader.records() {
        let record = result?;

        // Check if this is a transaction row (has Date filled)
        let date_str = record.get(date_idx).unwrap_or("").trim();
        let symbol_str = record.get(symbol_idx).unwrap_or("").trim();

        if !date_str.is_empty() && !symbol_str.is_empty() {
            // This is a transaction/lapse row - save the lapse date and symbol
            let lapse_date = NaiveDate::parse_from_str(date_str, "%m/%d/%Y")
                .map_err(|_| ConvertError::InvalidDate(date_str.to_string()))?;
            current_lapse = Some((symbol_str.to_string(), lapse_date));
        } else {
            // This is an award details row - extract FMV and associate with lapse date
            let fmv_str = record.get(fmv_idx).unwrap_or("").trim();

            if !fmv_str.is_empty()
                && let Some((symbol, lapse_date)) = &current_lapse
            {
                let price = parse_price(fmv_str)?;
                fmv_map.insert((symbol.clone(), *lapse_date), price);
            }
            // Don't reset current_lapse - multiple award rows may follow one transaction row
        }
    }

    Ok(AwardsData { fmv_map })
}

/// Find column index by name
fn find_column_index(headers: &StringRecord, name: &str) -> Result<usize, ConvertError> {
    headers
        .iter()
        .position(|h| h == name)
        .ok_or_else(|| ConvertError::MissingColumn(name.to_string()))
}

/// Parse price with $ prefix and commas
fn parse_price(price_str: &str) -> Result<Decimal, ConvertError> {
    let cleaned = price_str.replace(['$', ','], "");
    cleaned
        .parse::<Decimal>()
        .map_err(|_| ConvertError::InvalidAmount(price_str.to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal_macros::dec;

    #[test]
    fn test_parse_awards_json() {
        let json = r#"{
            "EquityAwards": [
                {
                    "Symbol": "XYZZ",
                    "EventDate": "04/25/2023",
                    "FairMarketValuePrice": "$125.6445"
                },
                {
                    "Symbol": "BAR",
                    "EventDate": "06/15/2023",
                    "FairMarketValuePrice": "340.50"
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
    fn test_fmv_lookback() {
        let json = r#"{
            "EquityAwards": [
                {
                    "Symbol": "XYZZ",
                    "EventDate": "04/25/2023",
                    "FairMarketValuePrice": "125.00"
                }
            ]
        }"#;

        let awards = parse_awards_json(json).unwrap();
        let vest_date = NaiveDate::from_ymd_opt(2023, 4, 25).unwrap();

        // Exact match - vest_date should equal query date
        let exact_date = NaiveDate::from_ymd_opt(2023, 4, 25).unwrap();
        let lookup = awards.get_fmv(&exact_date, "XYZZ").unwrap();
        assert_eq!(lookup.fmv, dec!(125.00));
        assert_eq!(lookup.vest_date, vest_date);

        // 3 days later (should find via lookback) - vest_date should be the original award date
        let later_date = NaiveDate::from_ymd_opt(2023, 4, 28).unwrap();
        let lookup = awards.get_fmv(&later_date, "XYZZ").unwrap();
        assert_eq!(lookup.fmv, dec!(125.00));
        assert_eq!(lookup.vest_date, vest_date); // Returns vest date, not query date

        // 8 days later (should fail - beyond lookback window)
        let too_late = NaiveDate::from_ymd_opt(2023, 5, 3).unwrap();
        assert!(awards.get_fmv(&too_late, "XYZZ").is_err());
    }

    #[test]
    fn test_missing_fmv() {
        let json = r#"{"EquityAwards": []}"#;
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
    fn test_parse_awards_csv() {
        // CSV format: transaction row (Date=lapse date, Symbol), then award details row (FMV)
        // The FMV is keyed by (symbol, lapse_date), NOT AwardDate
        let csv = r#"Date,Action,Symbol,Description,Quantity,FeesAndCommissions,DisbursementElection,Amount,AwardDate,AwardId,FairMarketValuePrice
04/25/2023,Lapse,XYZZ,RSU VEST,67.2,,,,,,,
,,,,,,,,01/15/2022,RSU-12345,$125.6445
06/15/2023,Lapse,BAR,RSU VEST,50,,,,,,,
,,,,,,,,02/20/2022,RSU-67890,$340.50
"#;

        let awards = parse_awards_csv(csv).unwrap();
        // Look up by LAPSE date, not award date
        let lapse_date1 = NaiveDate::from_ymd_opt(2023, 4, 25).unwrap();
        let lapse_date2 = NaiveDate::from_ymd_opt(2023, 6, 15).unwrap();

        let lookup1 = awards.get_fmv(&lapse_date1, "XYZZ").unwrap();
        assert_eq!(lookup1.fmv, dec!(125.6445));
        assert_eq!(lookup1.vest_date, lapse_date1);

        let lookup2 = awards.get_fmv(&lapse_date2, "BAR").unwrap();
        assert_eq!(lookup2.fmv, dec!(340.50));
        assert_eq!(lookup2.vest_date, lapse_date2);
    }

    #[test]
    fn test_parse_awards_csv_with_commas() {
        let csv = r#"Date,Action,Symbol,Description,Quantity,FeesAndCommissions,DisbursementElection,Amount,AwardDate,AwardId,FairMarketValuePrice
04/25/2023,Lapse,XYZZ,RSU VEST,67.2,,,,,,,
,,,,,,,,01/15/2022,RSU-12345,"$1,125.50"
"#;

        let awards = parse_awards_csv(csv).unwrap();
        let lapse_date = NaiveDate::from_ymd_opt(2023, 4, 25).unwrap();

        let lookup = awards.get_fmv(&lapse_date, "XYZZ").unwrap();
        assert_eq!(lookup.fmv, dec!(1125.50));
        assert_eq!(lookup.vest_date, lapse_date);
    }

    #[test]
    fn test_parse_awards_csv_missing_column() {
        let csv = r#"Date,Action,Symbol,Description
04/25/2023,Lapse,XYZZ,RSU VEST
"#;

        let result = parse_awards_csv(csv);
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            ConvertError::MissingColumn(_)
        ));
    }

    #[test]
    fn test_parse_awards_csv_invalid_date() {
        // Invalid date format in the Date column (lapse date)
        let csv = r#"Date,Action,Symbol,Description,Quantity,FeesAndCommissions,DisbursementElection,Amount,AwardDate,AwardId,FairMarketValuePrice
2023-04-25,Lapse,XYZZ,RSU VEST,67.2,,,,,,,
,,,,,,,,01/15/2022,RSU-12345,$125.50
"#;

        let result = parse_awards_csv(csv);
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), ConvertError::InvalidDate(_)));
    }

    #[test]
    fn test_parse_awards_csv_invalid_price() {
        let csv = r#"Date,Action,Symbol,Description,Quantity,FeesAndCommissions,DisbursementElection,Amount,AwardDate,AwardId,FairMarketValuePrice
04/25/2023,Lapse,XYZZ,RSU VEST,67.2,,,,,,,
,,,,,,,,01/15/2022,RSU-12345,INVALID
"#;

        let result = parse_awards_csv(csv);
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            ConvertError::InvalidAmount(_)
        ));
    }

    #[test]
    fn test_parse_awards_via_enum() {
        let json = r#"{"EquityAwards": [{"Symbol": "XYZZ", "EventDate": "04/25/2023", "FairMarketValuePrice": "$125.00"}]}"#;
        let csv = r#"Date,Action,Symbol,Description,Quantity,FeesAndCommissions,DisbursementElection,Amount,AwardDate,AwardId,FairMarketValuePrice
04/25/2023,Lapse,XYZZ,RSU VEST,67.2,,,,,,,
,,,,,,,,01/15/2022,RSU-12345,$125.00
"#;

        let awards_json = parse_awards(json, AwardsFormat::Json).unwrap();
        let awards_csv = parse_awards(csv, AwardsFormat::Csv).unwrap();

        let date = NaiveDate::from_ymd_opt(2023, 4, 25).unwrap();
        assert_eq!(
            awards_json.get_fmv(&date, "XYZZ").unwrap().fmv,
            dec!(125.00)
        );
        assert_eq!(awards_csv.get_fmv(&date, "XYZZ").unwrap().fmv, dec!(125.00));
    }

    // ===========================================
    // CSV Awards Corner Cases
    // ===========================================

    #[test]
    fn test_csv_multiple_awards_same_lapse_date() {
        // Multiple RSU grants vesting on the same day (common scenario)
        // Each grant may have different FMV based on when it was awarded
        // The last FMV for a given (symbol, lapse_date) wins
        let csv = r#"Date,Action,Symbol,Description,Quantity,FeesAndCommissions,DisbursementElection,Amount,AwardDate,AwardId,FairMarketValuePrice
04/25/2023,Lapse,XYZZ,RSU VEST,100,,,,,,,
,,,,,,,,01/15/2021,RSU-11111,$120.00
04/25/2023,Lapse,XYZZ,RSU VEST,50,,,,,,,
,,,,,,,,07/15/2021,RSU-22222,$125.00
04/25/2023,Lapse,XYZZ,RSU VEST,25,,,,,,,
,,,,,,,,01/15/2022,RSU-33333,$130.00
"#;

        let awards = parse_awards_csv(csv).unwrap();
        let lapse_date = NaiveDate::from_ymd_opt(2023, 4, 25).unwrap();

        // Last FMV for this date should be used
        let lookup = awards.get_fmv(&lapse_date, "XYZZ").unwrap();
        assert_eq!(lookup.fmv, dec!(130.00));
        assert_eq!(lookup.vest_date, lapse_date);
    }

    #[test]
    fn test_csv_multiple_symbols_same_day() {
        // Different symbols vesting on the same day
        let csv = r#"Date,Action,Symbol,Description,Quantity,FeesAndCommissions,DisbursementElection,Amount,AwardDate,AwardId,FairMarketValuePrice
04/25/2023,Lapse,XYZZ,RSU VEST,67.2,,,,,,,
,,,,,,,,01/15/2022,RSU-12345,$125.00
04/25/2023,Lapse,BAR,RSU VEST,50,,,,,,,
,,,,,,,,02/20/2022,RSU-67890,$340.50
04/25/2023,Lapse,FOO,RSU VEST,30,,,,,,,
,,,,,,,,03/15/2022,RSU-11111,$175.25
"#;

        let awards = parse_awards_csv(csv).unwrap();
        let lapse_date = NaiveDate::from_ymd_opt(2023, 4, 25).unwrap();

        assert_eq!(
            awards.get_fmv(&lapse_date, "XYZZ").unwrap().fmv,
            dec!(125.00)
        );
        assert_eq!(
            awards.get_fmv(&lapse_date, "BAR").unwrap().fmv,
            dec!(340.50)
        );
        assert_eq!(
            awards.get_fmv(&lapse_date, "FOO").unwrap().fmv,
            dec!(175.25)
        );
    }

    #[test]
    fn test_csv_empty_rows_between_records() {
        // CSV with extra empty rows (sometimes exported with blank lines)
        let csv = r#"Date,Action,Symbol,Description,Quantity,FeesAndCommissions,DisbursementElection,Amount,AwardDate,AwardId,FairMarketValuePrice
04/25/2023,Lapse,XYZZ,RSU VEST,67.2,,,,,,,
,,,,,,,,01/15/2022,RSU-12345,$125.00
,,,,,,,,,,,
06/15/2023,Lapse,BAR,RSU VEST,50,,,,,,,
,,,,,,,,02/20/2022,RSU-67890,$340.50
"#;

        let awards = parse_awards_csv(csv).unwrap();
        let date1 = NaiveDate::from_ymd_opt(2023, 4, 25).unwrap();
        let date2 = NaiveDate::from_ymd_opt(2023, 6, 15).unwrap();

        assert_eq!(awards.get_fmv(&date1, "XYZZ").unwrap().fmv, dec!(125.00));
        assert_eq!(awards.get_fmv(&date2, "BAR").unwrap().fmv, dec!(340.50));
    }

    #[test]
    fn test_csv_variable_field_count() {
        // Real exports sometimes have inconsistent field counts
        let csv = r#"Date,Action,Symbol,Description,Quantity,FeesAndCommissions,DisbursementElection,Amount,AwardDate,AwardId,FairMarketValuePrice
04/25/2023,Lapse,XYZZ,RSU VEST,67.2,,,,,,
,,,,,,,,01/15/2022,RSU-12345,$125.00,extra,fields,here
"#;

        let awards = parse_awards_csv(csv).unwrap();
        let lapse_date = NaiveDate::from_ymd_opt(2023, 4, 25).unwrap();
        assert_eq!(
            awards.get_fmv(&lapse_date, "XYZZ").unwrap().fmv,
            dec!(125.00)
        );
    }

    #[test]
    fn test_csv_fmv_without_dollar_sign() {
        // FMV price without $ prefix
        let csv = r#"Date,Action,Symbol,Description,Quantity,FeesAndCommissions,DisbursementElection,Amount,AwardDate,AwardId,FairMarketValuePrice
04/25/2023,Lapse,XYZZ,RSU VEST,67.2,,,,,,,
,,,,,,,,01/15/2022,RSU-12345,125.6445
"#;

        let awards = parse_awards_csv(csv).unwrap();
        let lapse_date = NaiveDate::from_ymd_opt(2023, 4, 25).unwrap();
        assert_eq!(
            awards.get_fmv(&lapse_date, "XYZZ").unwrap().fmv,
            dec!(125.6445)
        );
    }

    #[test]
    fn test_csv_quoted_fields() {
        // Fields wrapped in quotes (standard CSV encoding)
        let csv = r#"Date,Action,Symbol,Description,Quantity,FeesAndCommissions,DisbursementElection,Amount,AwardDate,AwardId,FairMarketValuePrice
"04/25/2023","Lapse","XYZZ","RSU VEST","67.2","","","","","",""
"","","","","","","","","01/15/2022","RSU-12345","$125.00"
"#;

        let awards = parse_awards_csv(csv).unwrap();
        let lapse_date = NaiveDate::from_ymd_opt(2023, 4, 25).unwrap();
        assert_eq!(
            awards.get_fmv(&lapse_date, "XYZZ").unwrap().fmv,
            dec!(125.00)
        );
    }

    #[test]
    fn test_csv_empty_awards_file() {
        // Awards file with only header, no data
        let csv = r#"Date,Action,Symbol,Description,Quantity,FeesAndCommissions,DisbursementElection,Amount,AwardDate,AwardId,FairMarketValuePrice
"#;

        let awards = parse_awards_csv(csv).unwrap();
        let date = NaiveDate::from_ymd_opt(2023, 4, 25).unwrap();
        assert!(awards.get_fmv(&date, "XYZZ").is_err());
    }

    #[test]
    fn test_csv_orphan_award_row() {
        // Award details row without preceding transaction row should be skipped
        let csv = r#"Date,Action,Symbol,Description,Quantity,FeesAndCommissions,DisbursementElection,Amount,AwardDate,AwardId,FairMarketValuePrice
,,,,,,,,01/15/2022,RSU-12345,$125.00
04/25/2023,Lapse,XYZZ,RSU VEST,67.2,,,,,,,
,,,,,,,,02/15/2022,RSU-67890,$130.00
"#;

        let awards = parse_awards_csv(csv).unwrap();
        let lapse_date = NaiveDate::from_ymd_opt(2023, 4, 25).unwrap();
        // Only the second FMV should be recorded (for the valid transaction row)
        assert_eq!(
            awards.get_fmv(&lapse_date, "XYZZ").unwrap().fmv,
            dec!(130.00)
        );
    }

    // ===========================================
    // FMV Lookback Edge Cases
    // ===========================================

    #[test]
    fn test_fmv_lookback_boundary_7_days() {
        // Test exact boundary of 7-day lookback
        let json = r#"{"EquityAwards": [{"Symbol": "XYZZ", "EventDate": "04/20/2023", "FairMarketValuePrice": "125.00"}]}"#;
        let awards = parse_awards_json(json).unwrap();
        let vest_date = NaiveDate::from_ymd_opt(2023, 4, 20).unwrap();

        // Day 0 (exact match)
        let day0 = NaiveDate::from_ymd_opt(2023, 4, 20).unwrap();
        let lookup = awards.get_fmv(&day0, "XYZZ").unwrap();
        assert_eq!(lookup.fmv, dec!(125.00));
        assert_eq!(lookup.vest_date, vest_date);

        // Day 7 (last valid lookback day) - vest_date should still be the original
        let day7 = NaiveDate::from_ymd_opt(2023, 4, 27).unwrap();
        let lookup = awards.get_fmv(&day7, "XYZZ").unwrap();
        assert_eq!(lookup.fmv, dec!(125.00));
        assert_eq!(lookup.vest_date, vest_date);

        // Day 8 (beyond lookback window)
        let day8 = NaiveDate::from_ymd_opt(2023, 4, 28).unwrap();
        assert!(awards.get_fmv(&day8, "XYZZ").is_err());
    }

    #[test]
    fn test_fmv_lookback_prefers_exact_match() {
        // If multiple dates could match, exact match should be preferred
        let json = r#"{"EquityAwards": [
            {"Symbol": "XYZZ", "EventDate": "04/20/2023", "FairMarketValuePrice": "120.00"},
            {"Symbol": "XYZZ", "EventDate": "04/25/2023", "FairMarketValuePrice": "125.00"}
        ]}"#;
        let awards = parse_awards_json(json).unwrap();

        let exact_date = NaiveDate::from_ymd_opt(2023, 4, 25).unwrap();
        // Should return 125.00 (exact match) not 120.00 (via lookback)
        let lookup = awards.get_fmv(&exact_date, "XYZZ").unwrap();
        assert_eq!(lookup.fmv, dec!(125.00));
        assert_eq!(lookup.vest_date, exact_date);
    }

    #[test]
    fn test_fmv_lookback_finds_closest() {
        // Lookback should find the most recent date within the window
        let json = r#"{"EquityAwards": [
            {"Symbol": "XYZZ", "EventDate": "04/20/2023", "FairMarketValuePrice": "120.00"},
            {"Symbol": "XYZZ", "EventDate": "04/23/2023", "FairMarketValuePrice": "123.00"}
        ]}"#;
        let awards = parse_awards_json(json).unwrap();

        let query_date = NaiveDate::from_ymd_opt(2023, 4, 25).unwrap();
        let expected_vest_date = NaiveDate::from_ymd_opt(2023, 4, 23).unwrap();
        // Should find 04/23 (2 days back) before 04/20 (5 days back)
        let lookup = awards.get_fmv(&query_date, "XYZZ").unwrap();
        assert_eq!(lookup.fmv, dec!(123.00));
        assert_eq!(lookup.vest_date, expected_vest_date);
    }

    #[test]
    fn test_fmv_case_sensitive_symbol() {
        // Symbol matching should be case-sensitive
        let json = r#"{"EquityAwards": [{"Symbol": "XYZZ", "EventDate": "04/25/2023", "FairMarketValuePrice": "125.00"}]}"#;
        let awards = parse_awards_json(json).unwrap();

        let date = NaiveDate::from_ymd_opt(2023, 4, 25).unwrap();
        assert_eq!(awards.get_fmv(&date, "XYZZ").unwrap().fmv, dec!(125.00));
        assert!(awards.get_fmv(&date, "goog").is_err());
        assert!(awards.get_fmv(&date, "Goog").is_err());
    }

    #[test]
    fn test_fmv_cross_month_lookback() {
        // Lookback should work across month boundaries
        let json = r#"{"EquityAwards": [{"Symbol": "XYZZ", "EventDate": "03/28/2023", "FairMarketValuePrice": "125.00"}]}"#;
        let awards = parse_awards_json(json).unwrap();

        let vest_date = NaiveDate::from_ymd_opt(2023, 3, 28).unwrap();
        // April 3 looking back to March 28 (6 days)
        let april_date = NaiveDate::from_ymd_opt(2023, 4, 3).unwrap();
        let lookup = awards.get_fmv(&april_date, "XYZZ").unwrap();
        assert_eq!(lookup.fmv, dec!(125.00));
        assert_eq!(lookup.vest_date, vest_date);
    }

    #[test]
    fn test_fmv_cross_year_lookback() {
        // Lookback should work across year boundaries
        let json = r#"{"EquityAwards": [{"Symbol": "XYZZ", "EventDate": "12/28/2022", "FairMarketValuePrice": "125.00"}]}"#;
        let awards = parse_awards_json(json).unwrap();

        let vest_date = NaiveDate::from_ymd_opt(2022, 12, 28).unwrap();
        // January 3, 2023 looking back to December 28, 2022 (6 days)
        let jan_date = NaiveDate::from_ymd_opt(2023, 1, 3).unwrap();
        let lookup = awards.get_fmv(&jan_date, "XYZZ").unwrap();
        assert_eq!(lookup.fmv, dec!(125.00));
        assert_eq!(lookup.vest_date, vest_date);
    }

    // ===========================================
    // JSON Awards Edge Cases
    // ===========================================

    #[test]
    fn test_json_empty_equity_awards_array() {
        let json = r#"{"EquityAwards": []}"#;
        let awards = parse_awards_json(json).unwrap();
        let date = NaiveDate::from_ymd_opt(2023, 4, 25).unwrap();
        assert!(awards.get_fmv(&date, "XYZZ").is_err());
    }

    #[test]
    fn test_json_fmv_various_formats() {
        // Test various FMV price formats
        let json = r#"{"EquityAwards": [
            {"Symbol": "A", "EventDate": "04/25/2023", "FairMarketValuePrice": "$125.00"},
            {"Symbol": "B", "EventDate": "04/25/2023", "FairMarketValuePrice": "125.00"},
            {"Symbol": "C", "EventDate": "04/25/2023", "FairMarketValuePrice": "$125.6445"},
            {"Symbol": "D", "EventDate": "04/25/2023", "FairMarketValuePrice": "0.50"}
        ]}"#;

        let awards = parse_awards_json(json).unwrap();
        let date = NaiveDate::from_ymd_opt(2023, 4, 25).unwrap();

        assert_eq!(awards.get_fmv(&date, "A").unwrap().fmv, dec!(125.00));
        assert_eq!(awards.get_fmv(&date, "B").unwrap().fmv, dec!(125.00));
        assert_eq!(awards.get_fmv(&date, "C").unwrap().fmv, dec!(125.6445));
        assert_eq!(awards.get_fmv(&date, "D").unwrap().fmv, dec!(0.50));
    }

    #[test]
    fn test_json_duplicate_entries_last_wins() {
        // If same symbol/date appears multiple times, last one wins
        let json = r#"{"EquityAwards": [
            {"Symbol": "XYZZ", "EventDate": "04/25/2023", "FairMarketValuePrice": "100.00"},
            {"Symbol": "XYZZ", "EventDate": "04/25/2023", "FairMarketValuePrice": "125.00"}
        ]}"#;

        let awards = parse_awards_json(json).unwrap();
        let date = NaiveDate::from_ymd_opt(2023, 4, 25).unwrap();
        assert_eq!(awards.get_fmv(&date, "XYZZ").unwrap().fmv, dec!(125.00));
    }
}
