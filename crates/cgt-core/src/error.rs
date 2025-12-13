use std::fmt;
use thiserror::Error;

/// Valid transaction types for CGT DSL
pub const VALID_TRANSACTION_TYPES: &[&str] =
    &["BUY", "SELL", "DIVIDEND", "CAPRETURN", "SPLIT", "UNSPLIT"];

#[derive(Error, Debug)]
pub enum CgtError {
    #[error("Parsing error: {0}")]
    ParseError(#[from] Box<pest::error::Error<crate::parser::Rule>>),

    #[error("{0}")]
    ParseErrorContext(ParseErrorContext),

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

/// Rich error context for parse errors with line numbers and suggestions.
#[derive(Debug, Clone)]
pub struct ParseErrorContext {
    /// Line number (1-indexed)
    pub line: usize,
    /// Column number (1-indexed)
    pub column: usize,
    /// What was found at the error location
    pub found: String,
    /// What was expected
    pub expected: String,
    /// Optional suggestion for fixing the error
    pub suggestion: Option<String>,
    /// The problematic line of input
    pub line_content: String,
}

impl ParseErrorContext {
    /// Create a new parse error context.
    pub fn new(
        line: usize,
        column: usize,
        found: String,
        expected: String,
        line_content: String,
    ) -> Self {
        Self {
            line,
            column,
            found,
            expected,
            suggestion: None,
            line_content,
        }
    }

    /// Add a suggestion to the error context.
    pub fn with_suggestion(mut self, suggestion: String) -> Self {
        self.suggestion = Some(suggestion);
        self
    }
}

impl fmt::Display for ParseErrorContext {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(
            f,
            "Parse error at line {}, column {}:",
            self.line, self.column
        )?;
        writeln!(f, "  |")?;
        writeln!(f, "{:3} | {}", self.line, self.line_content)?;

        // Create pointer to error location
        let pointer = format!("{:3} | {}^", "", " ".repeat(self.column.saturating_sub(1)));
        writeln!(f, "{}", pointer)?;

        writeln!(f, "  |")?;
        writeln!(f, "  = found: {}", self.found)?;
        writeln!(f, "  = expected: {}", self.expected)?;

        if let Some(ref suggestion) = self.suggestion {
            writeln!(f, "  = suggestion: {}", suggestion)?;
        }

        Ok(())
    }
}

/// Calculate Levenshtein distance between two strings.
/// Used for suggesting similar valid transaction types.
pub fn levenshtein_distance(a: &str, b: &str) -> usize {
    let a = a.to_uppercase();
    let b = b.to_uppercase();
    let a_chars: Vec<char> = a.chars().collect();
    let b_chars: Vec<char> = b.chars().collect();

    let m = a_chars.len();
    let n = b_chars.len();

    if m == 0 {
        return n;
    }
    if n == 0 {
        return m;
    }

    let mut matrix = vec![vec![0usize; n + 1]; m + 1];

    for (i, row) in matrix.iter_mut().enumerate().take(m + 1) {
        row[0] = i;
    }
    for (j, cell) in matrix[0].iter_mut().enumerate().take(n + 1) {
        *cell = j;
    }

    for i in 1..=m {
        for j in 1..=n {
            let cost = if a_chars[i - 1] == b_chars[j - 1] {
                0
            } else {
                1
            };
            matrix[i][j] = (matrix[i - 1][j] + 1)
                .min(matrix[i][j - 1] + 1)
                .min(matrix[i - 1][j - 1] + cost);
        }
    }

    matrix[m][n]
}

/// Suggest a valid transaction type based on input.
/// Returns the closest match if within edit distance of 3.
pub fn suggest_transaction_type(input: &str) -> Option<&'static str> {
    let input_upper = input.to_uppercase();

    // First check for exact match
    for &valid in VALID_TRANSACTION_TYPES {
        if valid == input_upper {
            return Some(valid);
        }
    }

    // Find closest match within threshold
    let mut best_match: Option<&'static str> = None;
    let mut best_distance = 4usize; // Threshold: only suggest if distance <= 3

    for &valid in VALID_TRANSACTION_TYPES {
        let distance = levenshtein_distance(&input_upper, valid);
        if distance < best_distance {
            best_distance = distance;
            best_match = Some(valid);
        }
    }

    best_match
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_levenshtein_distance() {
        assert_eq!(levenshtein_distance("", ""), 0);
        assert_eq!(levenshtein_distance("abc", "abc"), 0);
        assert_eq!(levenshtein_distance("abc", ""), 3);
        assert_eq!(levenshtein_distance("", "abc"), 3);
        assert_eq!(levenshtein_distance("buy", "BUY"), 0);
        assert_eq!(levenshtein_distance("BUUY", "BUY"), 1);
        assert_eq!(levenshtein_distance("BY", "BUY"), 1);
        assert_eq!(levenshtein_distance("SEEL", "SELL"), 1);
    }

    #[test]
    fn test_suggest_transaction_type() {
        assert_eq!(suggest_transaction_type("BUY"), Some("BUY"));
        assert_eq!(suggest_transaction_type("buy"), Some("BUY"));
        assert_eq!(suggest_transaction_type("BUUY"), Some("BUY"));
        assert_eq!(suggest_transaction_type("BY"), Some("BUY"));
        assert_eq!(suggest_transaction_type("SEEL"), Some("SELL"));
        assert_eq!(suggest_transaction_type("DIVIDND"), Some("DIVIDEND"));
        assert_eq!(suggest_transaction_type("XXXXX"), None);
    }

    #[test]
    fn test_parse_error_context_display() {
        let ctx = ParseErrorContext::new(
            5,
            10,
            "BUUY".to_string(),
            "transaction type".to_string(),
            "2024-01-01 BUUY AAPL 100 150.00".to_string(),
        )
        .with_suggestion("Did you mean 'BUY'?".to_string());

        let display = format!("{}", ctx);
        assert!(display.contains("line 5"));
        assert!(display.contains("column 10"));
        assert!(display.contains("BUUY"));
        assert!(display.contains("Did you mean 'BUY'?"));
    }
}
