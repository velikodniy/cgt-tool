//! CGT engine: parsing, validation, FX, matching, and report building.

pub mod config;
pub mod dsl;
pub mod error;
pub mod model;
pub mod money;
pub mod validate;

// Compiled only for tests: no non-test code consumes the engine yet, so a
// non-test build would flag the whole module as dead code under
// `-D warnings`.
#[cfg(test)]
mod engine;

pub use config::Config;
pub use error::CgtError;
pub use model::{Operation, TaxPeriod, Transaction};
pub use validate::{ValidationError, ValidationResult, ValidationWarning, validate};
