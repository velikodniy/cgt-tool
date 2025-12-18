use pest_consume::Error as PestConsumeError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum CgtError {
    #[error("Parsing error: {0}")]
    ParseError(#[from] Box<pest::error::Error<crate::parser::Rule>>),

    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("Serialization error: {0}")]
    SerializationError(#[from] serde_json::Error),

    #[error("Invalid transaction: {0}")]
    InvalidTransaction(String),

    #[error("Invalid date: year {year} is out of valid range")]
    InvalidDateYear { year: i32 },

    #[error("Invalid tax year: {0} is out of valid range (1900-2100)")]
    InvalidTaxYear(u16),

    #[error("Unsupported tax year {0} for CGT exemption lookup - please update the tool")]
    UnsupportedExemptionYear(u16),

    #[error("PDF generation failed: {0}")]
    PdfGeneration(String),

    #[error("Invalid currency code '{code}': not a recognized ISO 4217 currency")]
    InvalidCurrencyCode { code: String },

    #[error("Missing FX rate for {currency} in {year}-{month:02}")]
    MissingFxRate {
        currency: String,
        year: i32,
        month: u32,
    },
}

impl From<PestConsumeError<crate::parser::Rule>> for CgtError {
    fn from(err: PestConsumeError<crate::parser::Rule>) -> Self {
        // Convert pest_consume error to ParseError
        // The error already contains line/column information
        CgtError::ParseError(Box::new(err.renamed_rules(|rule| format!("{:?}", rule))))
    }
}
