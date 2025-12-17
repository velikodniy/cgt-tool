//! Error types for the MCP server.

use thiserror::Error;

/// Errors that can occur in the MCP server.
#[derive(Debug, Error)]
pub enum McpServerError {
    /// Error from cgt-core during parsing or calculation.
    #[error("CGT error: {0}")]
    Cgt(#[from] cgt_core::CgtError),

    /// Error from cgt-money during FX operations.
    #[error("FX error: {0}")]
    Fx(#[from] cgt_money::FxLoaderError),

    /// Error during JSON serialization.
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    /// MCP service error.
    #[error("MCP service error: {0}")]
    Service(String),

    /// Invalid input parameter.
    #[error("Invalid parameter: {0}")]
    InvalidParameter(String),

    /// Resource not found.
    #[error("Resource not found: {0}")]
    ResourceNotFound(String),

    /// Disposal not found for explain_matching.
    #[error("No disposal found for {ticker} on {date}")]
    DisposalNotFound { ticker: String, date: String },
}
