//! CGT engine: parsing, validation, FX, matching, and report building.

pub mod config;
pub mod dsl;
pub mod error;
pub mod model;
pub mod money;
pub mod validate;

// Engine internals (Milestone C: normalize + plan). Compiled only for tests
// until Milestone D wires the public `calculate` entry through them; in a
// non-test build the module would be entirely dead code, and CI lints with
// `-D warnings`. Milestone D removes this cfg.
#[cfg(test)]
mod engine;

pub use config::Config;
pub use error::CgtError;
pub use model::{Operation, TaxPeriod, Transaction};
pub use validate::{ValidationError, ValidationResult, ValidationWarning, validate};
