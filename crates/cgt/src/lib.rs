//! CGT engine: parsing, validation, FX, matching, and report building.

pub mod dsl;
pub mod error;
pub mod model;
pub mod money;

pub use error::CgtError;
pub use model::{Operation, TaxPeriod, Transaction};
