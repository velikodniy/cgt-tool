//! Exact share quantities across splits and partial matches.

use num_bigint::BigInt;
use num_rational::BigRational;
use num_traits::{Signed, ToPrimitive, Zero};
use rust_decimal::Decimal;

use crate::error::CgtError;

#[derive(Debug, Clone, Default, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) struct Quantity(BigRational);

impl From<Decimal> for Quantity {
    fn from(value: Decimal) -> Self {
        Self(BigRational::new(
            BigInt::from(value.mantissa()),
            BigInt::from(10_u8).pow(value.scale()),
        ))
    }
}

impl PartialEq<Decimal> for Quantity {
    fn eq(&self, other: &Decimal) -> bool {
        self == &Self::from(*other)
    }
}

impl PartialOrd<Decimal> for Quantity {
    fn partial_cmp(&self, other: &Decimal) -> Option<std::cmp::Ordering> {
        Some(self.cmp(&Self::from(*other)))
    }
}

impl Quantity {
    pub(crate) fn is_positive(&self) -> bool {
        self.0.is_positive()
    }

    pub(crate) fn is_zero(&self) -> bool {
        self.0.is_zero()
    }

    pub(crate) fn is_integer(&self) -> bool {
        self.0.is_integer()
    }

    pub(crate) fn add(&mut self, other: &Self) {
        self.0 += &other.0;
    }

    pub(crate) fn subtract(&mut self, other: &Self) {
        self.0 -= &other.0;
    }

    pub(crate) fn multiplied(&self, other: &Self) -> Self {
        Self(&self.0 * &other.0)
    }

    pub(crate) fn divided(&self, other: &Self) -> Option<Self> {
        if other.is_zero() {
            None
        } else {
            Some(Self(&self.0 / &other.0))
        }
    }

    /// Allocate a monetary total in this exact share proportion.
    pub(crate) fn proportion_of(&self, total: &Self, amount: Decimal) -> Result<Decimal, CgtError> {
        let share = self
            .divided(total)
            .ok_or_else(|| CgtError::ArithmeticOverflow {
                operation: "allocation with zero quantity".into(),
            })?;
        share.multiplied(&Self::from(amount)).decimal_value()
    }

    /// Convert only at a decimal API boundary, using midpoint-away-from-zero.
    pub(crate) fn decimal(&self) -> Result<Decimal, CgtError> {
        let value = self.decimal_value()?;
        if value.is_zero() && !self.is_zero() {
            return Err(CgtError::ArithmeticOverflow {
                operation: "share quantity below supported decimal precision".into(),
            });
        }
        Ok(value)
    }

    fn decimal_value(&self) -> Result<Decimal, CgtError> {
        let negative = self.0.is_negative();
        let numerator = self.0.numer().abs();
        let denominator = self.0.denom();
        let max = BigInt::from(Decimal::MAX.mantissa());
        if numerator > &max * denominator {
            return Err(CgtError::ArithmeticOverflow {
                operation: "decimal conversion".into(),
            });
        }
        for scale in (0..=28).rev() {
            let scaled = &numerator * BigInt::from(10_u8).pow(scale);
            let mut mantissa = &scaled / denominator;
            if (&scaled % denominator) * 2_u8 >= *denominator {
                mantissa += 1_u8;
            }
            if mantissa <= max
                && let Some(mut value) = mantissa.to_i128()
            {
                if negative {
                    value = -value;
                }
                return Ok(Decimal::from_i128_with_scale(value, scale).normalize());
            }
        }
        Err(CgtError::ArithmeticOverflow {
            operation: "share quantity conversion".into(),
        })
    }
}
