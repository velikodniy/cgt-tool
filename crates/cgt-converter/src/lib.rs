pub mod error;
#[macro_use]
mod string_enum;

pub mod output;
pub mod schwab;

pub use error::ConvertError;

/// Output from a broker conversion operation
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConvertOutput {
    /// The generated CGT DSL content
    pub cgt_content: String,
    /// Warnings generated during conversion (e.g., skipped transactions)
    pub warnings: Vec<String>,
    /// Number of transactions skipped (not CGT-relevant)
    pub skipped_count: usize,
}

/// Trait for converting broker-specific export formats to CGT DSL
pub trait BrokerConverter {
    /// Broker-specific input type (e.g., CSV string, JSON string, multiple files)
    type Input;

    /// Convert broker export(s) to CGT DSL format
    ///
    /// # Errors
    ///
    /// Returns `ConvertError` if:
    /// - Input format is invalid (CSV/JSON parsing errors)
    /// - Required data is missing (columns, dates, prices)
    /// - Data cannot be converted to CGT format
    fn convert(&self, input: &Self::Input) -> Result<ConvertOutput, ConvertError>;

    /// Broker identifier for CLI/logging
    fn broker_name(&self) -> &'static str;
}
