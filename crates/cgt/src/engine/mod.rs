//! Private engine internals. Milestone C: normalization and quantity-only
//! match planning. Valuation and report building arrive in Milestone D.

pub(crate) mod normalize;
pub(crate) mod plan;

#[cfg(test)]
mod fixture_tests;
