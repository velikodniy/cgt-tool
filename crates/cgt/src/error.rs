//! Engine error type. Variants are added as engine milestones land.

use pest_consume::Error as PestConsumeError;
use thiserror::Error;

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

    #[error("Configuration error: {0}")]
    ConfigError(String),
}

impl From<PestConsumeError<crate::dsl::Rule>> for CgtError {
    fn from(err: PestConsumeError<crate::dsl::Rule>) -> Self {
        // Convert pest_consume error to ParseError
        // The error already contains line/column information
        CgtError::ParseError(Box::new(err.renamed_rules(|rule| format!("{:?}", rule))))
    }
}
