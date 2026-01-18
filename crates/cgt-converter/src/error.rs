use thiserror::Error;

#[derive(Debug, Error)]
pub enum ConvertError {
    #[error("JSON parsing error: {0}")]
    JsonError(#[from] serde_json::Error),

    #[error("Invalid date format: {0}")]
    InvalidDate(String),

    #[error("Invalid amount format: {0}")]
    InvalidAmount(String),

    #[error("Missing fair market value for Stock Plan Activity on {date} for {symbol}")]
    MissingFairMarketValue { date: String, symbol: String },

    #[error("Invalid transaction: {0}")]
    InvalidTransaction(String),
}
