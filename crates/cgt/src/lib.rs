//! CGT engine: parsing, validation, FX, matching, and report building.

pub mod config;
pub mod dsl;
pub mod error;
pub mod model;
pub mod money;
pub mod validate;

pub use config::Config;
pub use error::CgtError;
pub use model::{Operation, TaxPeriod, Transaction};
pub use validate::{ValidationError, ValidationResult, ValidationWarning, validate};
