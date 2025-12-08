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

    #[error("Unexpected parser state: expected {expected}")]
    UnexpectedParserState { expected: &'static str },

    #[error("Invalid date: year {year} is out of valid range")]
    InvalidDateYear { year: i32 },
}
