mod amount;
mod fx;

pub use amount::{CurrencyAmount, FxConversionError};
pub use fx::FxRates;
pub use iso_currency::Currency;

/// Convert a checked decimal operation into a calculation error.
pub(crate) fn checked(
    value: Option<rust_decimal::Decimal>,
    operation: impl Into<String>,
) -> Result<rust_decimal::Decimal, crate::CgtError> {
    value.ok_or_else(|| crate::CgtError::ArithmeticOverflow {
        operation: operation.into(),
    })
}
