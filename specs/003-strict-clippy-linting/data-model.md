# Data Model: Strict Clippy Linting

**Feature**: 003-strict-clippy-linting
**Date**: 2025-12-08

## Entity Changes

This feature extends the existing `CgtError` enum with new variants for proper error handling.

### CgtError (Extended)

**Location**: `crates/cgt-core/src/error.rs`

```rust
use thiserror::Error;

#[derive(Error, Debug)]
pub enum CgtError {
    // === Existing variants (unchanged) ===
    #[error("Parsing error: {0}")]
    ParseError(#[from] Box<pest::error::Error<crate::parser::Rule>>),

    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("Serialization error: {0}")]
    SerializationError(#[from] serde_json::Error),

    #[error("Invalid transaction: {0}")]
    InvalidTransaction(String),

    // === New variants for unwrap replacement ===
    #[error("Unexpected parser state: expected {expected}")]
    UnexpectedParserState { expected: &'static str },

    #[error("Invalid date: year {year} is out of valid range")]
    InvalidDateYear { year: i32 },
}
```

### Variant Descriptions

| Variant                 | Purpose                                             | Used When                                                     |
| ----------------------- | --------------------------------------------------- | ------------------------------------------------------------- |
| `UnexpectedParserState` | Replace `.unwrap()` on `Option` from pest iterators | Grammar guarantees token exists but we need graceful fallback |
| `InvalidDateYear`       | Replace `.unwrap()` on date creation                | Tax year calculation with out-of-range year                   |

## New Configuration File

### clippy.toml

**Location**: Workspace root `/clippy.toml`

```toml
# Workspace-level Clippy configuration
# Strict linting - no panicking code in production

[lints.clippy]
unwrap_used = "deny"
expect_used = "deny"
panic = "deny"
todo = "deny"
unimplemented = "deny"
```

**Purpose**: Centralized lint configuration for all crates in workspace.

## Design Rationale

### Why New Error Variants vs Reusing InvalidTransaction

1. **Type Safety**: Structured variants allow pattern matching on error types
2. **Clarity**: Each variant clearly indicates the error source
3. **Static Strings**: `&'static str` avoids allocation for internal errors
4. **Separation**: Parser state errors are internal; user-facing errors use `InvalidTransaction`

### Why clippy.toml vs Cargo.toml [lints]

1. **Compatibility**: Works with all Rust versions
2. **Flexibility**: Clippy-specific options available
3. **Separation**: Lint config separate from build config
4. **Discoverability**: Standard location for Clippy configuration

### Test Code Exemptions

Tests use `#[allow(clippy::expect_used)]` at module level because:

- Panicking in tests is expected behavior (test failure)
- `.expect()` provides better failure messages than `.unwrap()`
- Full `Result` handling in tests adds noise

## Backward Compatibility

- Existing `CgtError` variants unchanged
- New variants are additive
- Public API unchanged - errors still implement `std::error::Error`
- All existing valid inputs continue to work
