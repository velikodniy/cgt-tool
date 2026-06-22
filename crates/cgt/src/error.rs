//! Engine error type.

use chrono::NaiveDate;
use pest_consume::Error as PestConsumeError;
use rust_decimal::Decimal;
use thiserror::Error;

use crate::money::FxConversionError;

#[derive(Debug, Error)]
pub enum CgtError {
    #[error("Parsing error: {0}")]
    ParseError(Box<pest::error::Error<crate::dsl::Rule>>),

    #[error("Invalid date: year {year} is out of valid range")]
    InvalidDateYear { year: i32 },

    #[error("Invalid tax year: {0} is out of valid range (1900-2100)")]
    InvalidTaxYear(u16),

    #[error("Unsupported tax year {0} for CGT exemption lookup - please update the tool")]
    UnsupportedExemptionYear(u16),

    #[error("Missing FX rate for {currency} in {year}-{month:02}")]
    MissingFxRate {
        currency: String,
        year: i32,
        month: u32,
    },

    #[error(
        "SELL {ticker} on {date}: disposal of {attempted} shares exceeds holding of {held} \
         (same-day ledger: {ledger}, S104 pool: {pool}). B&B determines cost basis for a \
         valid disposal; it does not enable disposing of shares not held (CG51590)."
    )]
    DisposalExceedsHolding {
        ticker: String,
        date: NaiveDate,
        attempted: Decimal,
        held: Decimal,
        ledger: Decimal,
        pool: Decimal,
    },

    #[error("SELL {ticker} on {date} has no prior acquisitions (attempted to dispose {attempted})")]
    NoPriorAcquisitions {
        ticker: String,
        date: NaiveDate,
        attempted: Decimal,
    },

    #[error(
        "SELL {ticker} on {date} exceeds holding: attempted {attempted}, matched {matched}, \
         unmatched {unmatched}"
    )]
    DisposalPartiallyUnmatched {
        ticker: String,
        date: NaiveDate,
        attempted: Decimal,
        matched: Decimal,
        unmatched: Decimal,
    },

    #[error("B&B reservation exceeds buy amount for {ticker} on {date}")]
    BnbReservationExceedsBuy { ticker: String, date: NaiveDate },

    #[error("Configuration error: {0}")]
    ConfigError(String),

    #[error(
        "CAPRETURN {ticker} on {date}: capital distribution £{net} exceeds allowable cost \
         £{basis}. TCGA92/S122(2) does not apply when distribution exceeds expenditure \
         (CG57847). Part-disposal under S122(1) or election under S122(4) is required."
    )]
    CapitalReturnExceedsBasis {
        ticker: String,
        date: NaiveDate,
        net: Decimal,
        basis: Decimal,
    },

    #[error(
        "ACCUMULATION {ticker} on {date}: no holding to credit (not yet bought or fully disposed)"
    )]
    AccumulationWithoutHolding { ticker: String, date: NaiveDate },

    #[error(
        "UNSPLIT {ticker} on {date}: holding of {holding} is not divisible by ratio {ratio} \
         (a consolidation must leave whole shares; CG51746)"
    )]
    UnsplitIndivisibleHolding {
        ticker: String,
        date: NaiveDate,
        holding: Decimal,
        ratio: Decimal,
    },

    #[error("{0}")]
    Validation(ValidationErrors),
}

/// One or more input validation errors, rendered one per line.
#[derive(Debug)]
pub struct ValidationErrors(pub Vec<crate::validate::ValidationError>);

impl std::fmt::Display for ValidationErrors {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for (i, err) in self.0.iter().enumerate() {
            if i > 0 {
                writeln!(f)?;
            }
            write!(f, "{err}")?;
        }
        Ok(())
    }
}

impl From<PestConsumeError<crate::dsl::Rule>> for CgtError {
    fn from(err: PestConsumeError<crate::dsl::Rule>) -> Self {
        // Convert pest_consume error to ParseError
        // The error already contains line/column information
        CgtError::ParseError(Box::new(err.renamed_rules(|rule| format!("{:?}", rule))))
    }
}

impl From<FxConversionError> for CgtError {
    fn from(err: FxConversionError) -> Self {
        match err {
            FxConversionError::MissingRate {
                currency,
                year,
                month,
            } => CgtError::MissingFxRate {
                currency,
                year,
                month,
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use chrono::NaiveDate;
    use rust_decimal_macros::dec;

    use super::CgtError;
    use crate::money::FxConversionError;

    fn date(y: i32, m: u32, d: u32) -> NaiveDate {
        NaiveDate::from_ymd_opt(y, m, d).expect("valid test date")
    }

    #[test]
    fn missing_fx_rate_display() {
        let err = CgtError::MissingFxRate {
            currency: "USD".to_string(),
            year: 2024,
            month: 2,
        };
        assert_eq!(err.to_string(), "Missing FX rate for USD in 2024-02");
    }

    #[test]
    fn fx_conversion_error_maps_to_missing_fx_rate() {
        let err = CgtError::from(FxConversionError::MissingRate {
            currency: "EUR".to_string(),
            year: 2023,
            month: 11,
        });
        assert!(matches!(
            err,
            CgtError::MissingFxRate { ref currency, year: 2023, month: 11 } if currency == "EUR"
        ));
    }

    #[test]
    fn disposal_exceeds_holding_display_carries_quantities_and_citation() {
        let err = CgtError::DisposalExceedsHolding {
            ticker: "SNAP".to_string(),
            date: date(2024, 2, 1),
            attempted: dec!(100),
            held: dec!(80),
            ledger: dec!(30),
            pool: dec!(50),
        };
        assert_eq!(
            err.to_string(),
            "SELL SNAP on 2024-02-01: disposal of 100 shares exceeds holding of 80 \
             (same-day ledger: 30, S104 pool: 50). B&B determines cost basis for a \
             valid disposal; it does not enable disposing of shares not held (CG51590)."
        );
    }

    #[test]
    fn no_prior_acquisitions_display() {
        let err = CgtError::NoPriorAcquisitions {
            ticker: "ABC".to_string(),
            date: date(2024, 2, 1),
            attempted: dec!(10),
        };
        assert_eq!(
            err.to_string(),
            "SELL ABC on 2024-02-01 has no prior acquisitions (attempted to dispose 10)"
        );
    }

    #[test]
    fn disposal_partially_unmatched_display() {
        let err = CgtError::DisposalPartiallyUnmatched {
            ticker: "ABC".to_string(),
            date: date(2024, 6, 1),
            attempted: dec!(10),
            matched: dec!(7),
            unmatched: dec!(3),
        };
        assert_eq!(
            err.to_string(),
            "SELL ABC on 2024-06-01 exceeds holding: attempted 10, matched 7, unmatched 3"
        );
    }

    #[test]
    fn bnb_reservation_guard_display() {
        let err = CgtError::BnbReservationExceedsBuy {
            ticker: "ABC".to_string(),
            date: date(2024, 3, 1),
        };
        assert_eq!(
            err.to_string(),
            "B&B reservation exceeds buy amount for ABC on 2024-03-01"
        );
    }

    #[test]
    fn capital_return_exceeds_basis_display_carries_citation() {
        let err = CgtError::CapitalReturnExceedsBasis {
            ticker: "ACME".to_string(),
            date: date(2024, 6, 15),
            net: dec!(150.50),
            basis: dec!(100.00),
        };
        let rendered = err.to_string();
        assert!(rendered.contains("CAPRETURN ACME on 2024-06-15"));
        assert!(rendered.contains("£150.50"));
        assert!(rendered.contains("£100.00"));
        assert!(rendered.contains("TCGA92/S122(2)"));
        assert!(rendered.contains("CG57847"));
    }

    #[test]
    fn validation_display_lists_one_error_per_line() {
        use crate::error::ValidationErrors;
        use crate::validate::ValidationError;

        let err = CgtError::Validation(ValidationErrors(vec![
            ValidationError {
                line: Some(1),
                date: date(2024, 1, 1),
                ticker: "ABC".to_string(),
                message: "first problem".to_string(),
            },
            ValidationError {
                line: None,
                date: date(2024, 2, 1),
                ticker: "XYZ".to_string(),
                message: "second problem".to_string(),
            },
        ]));
        assert_eq!(
            err.to_string(),
            "Error (line 1): ABC on 2024-01-01 - first problem\n\
             Error: XYZ on 2024-02-01 - second problem"
        );
    }
}
