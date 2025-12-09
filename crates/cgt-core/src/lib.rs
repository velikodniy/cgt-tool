pub mod calculator;
pub mod config;
pub mod error;
pub mod exemption;
pub mod formatting;
pub mod matcher;
pub mod models;
pub mod parser;
pub mod validation;

pub use config::Config;
pub use error::CgtError;
pub use exemption::get_exemption;
pub use models::*;
pub use validation::{ValidationError, ValidationResult, ValidationWarning, validate};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tax_period_serialization() {
        let period = TaxPeriod::new(2023).expect("valid year");
        let json = serde_json::to_string(&period).expect("serialize");
        assert_eq!(json, "\"2023/24\"");
    }

    #[test]
    fn test_tax_period_deserialization_valid() {
        let period: TaxPeriod = serde_json::from_str("\"2023/24\"").expect("deserialize");
        assert_eq!(period.start_year(), 2023);
        assert_eq!(period.end_year(), 2024);
    }

    #[test]
    fn test_tax_period_deserialization_invalid_years() {
        // 2023/27 is invalid - end year should be 24, not 27
        let result: Result<TaxPeriod, _> = serde_json::from_str("\"2023/27\"");
        assert!(result.is_err());
        let err = result.unwrap_err().to_string();
        assert!(
            err.contains("consecutive"),
            "Expected consecutive year error, got: {}",
            err
        );
    }

    #[test]
    fn test_tax_period_from_date() {
        use chrono::NaiveDate;

        // March 15, 2024 is in tax year 2023/24 (before April 6)
        let date = NaiveDate::from_ymd_opt(2024, 3, 15).expect("valid date");
        let period = TaxPeriod::from_date(date);
        assert_eq!(period.start_year(), 2023);

        // April 10, 2024 is in tax year 2024/25 (on or after April 6)
        let date = NaiveDate::from_ymd_opt(2024, 4, 10).expect("valid date");
        let period = TaxPeriod::from_date(date);
        assert_eq!(period.start_year(), 2024);

        // April 5, 2024 is still in tax year 2023/24 (before April 6)
        let date = NaiveDate::from_ymd_opt(2024, 4, 5).expect("valid date");
        let period = TaxPeriod::from_date(date);
        assert_eq!(period.start_year(), 2023);

        // April 6, 2024 is in tax year 2024/25 (on April 6)
        let date = NaiveDate::from_ymd_opt(2024, 4, 6).expect("valid date");
        let period = TaxPeriod::from_date(date);
        assert_eq!(period.start_year(), 2024);
    }
}
