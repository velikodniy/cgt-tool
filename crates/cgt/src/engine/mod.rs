//! Private engine internals: normalization and quantity-only match planning.

pub(crate) mod normalize;
pub(crate) mod plan;
pub(crate) mod value;

#[cfg(test)]
mod fixture_tests;
