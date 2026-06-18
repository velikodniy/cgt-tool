//! Private engine internals: normalization, quantity-only match planning,
//! valuation, and report assembly.

pub(crate) mod normalize;
pub(crate) mod plan;
pub(crate) mod report;
pub(crate) mod value;

#[cfg(test)]
mod fixture_tests;
#[cfg(test)]
mod fixture_value_tests;
