use thiserror::Error;

#[derive(Debug, Error)]
pub enum ConvertError {
    #[error("CSV parsing error: {0}")]
    CsvError(#[from] csv::Error),

    #[error("JSON parsing error: {0}")]
    JsonError(#[from] serde_json::Error),

    #[error("Invalid date format: {0}")]
    InvalidDate(String),

    #[error("Invalid amount format: {0}")]
    InvalidAmount(String),

    #[error("Missing required column: {0}")]
    MissingColumn(String),

    #[error("Missing fair market value for Stock Plan Activity on {date} for {symbol}")]
    MissingFairMarketValue { date: String, symbol: String },

    #[error("Invalid transaction: {0}")]
    InvalidTransaction(String),
}
