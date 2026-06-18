//! CGT engine: parsing, validation, FX, matching, and report building.

pub mod config;
pub mod dsl;
pub mod error;
pub mod format;
pub mod model;
pub mod money;
pub mod report;
pub mod validate;

mod engine;

pub use config::Config;
pub use error::CgtError;
pub use format::plain;
pub use model::{Operation, TaxPeriod, Transaction};
pub use money::FxCache;
pub use report::{Disposal, Holding, MatchLeg, MatchRule, TaxReport, TaxYearSummary};
pub use validate::{ValidationError, ValidationResult, ValidationWarning, validate};

/// Calculate the CGT report for a list of transactions.
///
/// Validation runs first; invalid input is rejected with
/// [`CgtError::Validation`] (warnings are not fatal). `tax_year_start` filters
/// to a single tax year when `Some`; `None` reports every disposal year.
/// `fx_cache` supplies FX rates for non-GBP amounts.
///
/// # Errors
/// Returns [`CgtError`] for validation failures, missing FX rates, unmatched
/// disposals, basis violations, or an unsupported exemption year.
pub fn calculate(
    transactions: &[Transaction],
    tax_year_start: Option<i32>,
    fx_cache: Option<&FxCache>,
    config: &Config,
) -> Result<TaxReport, CgtError> {
    let validation = validate::validate(transactions);
    if !validation.is_valid() {
        return Err(CgtError::Validation(error::ValidationErrors(
            validation.errors,
        )));
    }
    let stream = engine::normalize::normalize(transactions, fx_cache)?;
    let plan = engine::plan::plan(&stream)?;
    let valued = engine::value::value(&stream, &plan)?;
    engine::report::build(
        &stream,
        &valued,
        &plan,
        tax_year_start,
        config,
        Some(transactions.to_vec()),
    )
}

#[cfg(test)]
mod tests {
    use chrono::NaiveDate;
    use rust_decimal::Decimal;
    use rust_decimal_macros::dec;

    use super::{CgtError, Config, MatchRule, Operation, Transaction, calculate};
    use crate::money::CurrencyAmount;

    fn config() -> Config {
        Config::embedded().expect("embedded config loads")
    }

    #[test]
    fn calculate_over_a_gbp_input_yields_a_disposal_and_a_holding() {
        // A pool buy plus a partial sell: one Section 104 disposal and a
        // residual holding survive into the report.
        let transactions = crate::dsl::parse(
            "2018-08-28 BUY GB00B41YBW71 10 @ 4.1565 FEES 12.5\n\
             2019-01-15 SELL GB00B41YBW71 4 @ 4.6702 FEES 5.0\n",
        )
        .expect("fixture parses");

        let report = calculate(&transactions, None, None, &config()).expect("report builds");

        assert_eq!(report.tax_years.len(), 1);
        let year = &report.tax_years[0];
        assert_eq!(year.period.start_year(), 2018);
        assert_eq!(year.disposal_count, 1);
        let disposal = &year.disposals[0];
        assert_eq!(disposal.ticker, "GB00B41YBW71");
        assert_eq!(disposal.quantity, dec!(4));
        assert_eq!(disposal.legs.len(), 1);
        assert_eq!(disposal.legs[0].rule, MatchRule::Section104);

        // The 6-share residue remains pooled and is reported as a holding.
        assert_eq!(report.holdings.len(), 1);
        assert_eq!(report.holdings[0].ticker, "GB00B41YBW71");
        assert_eq!(report.holdings[0].quantity, dec!(6));

        // The input echo is populated for downstream renderers.
        assert_eq!(
            report.transactions.as_ref().map(Vec::len),
            Some(transactions.len())
        );
    }

    #[test]
    fn calculate_rejects_invalid_input_with_a_validation_error() {
        // A zero-quantity BUY is not constructible via the DSL (model
        // validation rejects it); built directly to drive the validation gate.
        let transactions = vec![Transaction {
            date: NaiveDate::from_ymd_opt(2024, 1, 1).expect("valid date"),
            ticker: "ABC".to_string(),
            operation: Operation::Buy {
                amount: Decimal::ZERO,
                price: CurrencyAmount::default(),
                fees: CurrencyAmount::default(),
            },
        }];

        let err = calculate(&transactions, None, None, &config())
            .expect_err("zero-quantity BUY must be rejected");
        assert!(matches!(err, CgtError::Validation(_)));
    }
}
