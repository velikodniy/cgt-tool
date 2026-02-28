use cgt_core::{Config, Disposal, calculator, parser, validate};
use cgt_money::load_default_cache;
use rust_decimal::Decimal;
use serde::Serialize;
use wasm_bindgen::prelude::*;

mod utils;
use utils::map_error;

/// Initialize the WASM module. This must be called before using any other functions.
///
/// # Example
/// ```javascript
/// import init, { parse_transactions } from './pkg/cgt_wasm.js';
///
/// await init();
/// const result = parse_transactions("BUY 2024-01-15 AAPL 10 @ 150.00 USD");
/// ```
#[wasm_bindgen(start)]
pub fn init() {
    #[cfg(feature = "console_error_panic_hook")]
    console_error_panic_hook::set_once();
}

/// Parse transaction DSL and return JSON representation of transactions.
///
/// # Arguments
/// * `dsl` - Transaction DSL string
///
/// # Returns
/// JSON array of parsed transactions
///
/// # Example
/// ```javascript
/// const dsl = `
///   BUY 2024-01-15 AAPL 10 @ 150.00 USD
///   SELL 2024-06-20 AAPL 5 @ 180.00 USD
/// `;
/// const transactions = JSON.parse(parse_transactions(dsl));
/// ```
#[wasm_bindgen]
pub fn parse_transactions(dsl: &str) -> Result<String, JsValue> {
    let transactions = parser::parse_file(dsl).map_err(map_error)?;
    serde_json::to_string_pretty(&transactions).map_err(map_error)
}

/// Tax year summary with calculated exemption and tax liability
#[derive(Serialize)]
struct TaxYear {
    period: Option<String>,
    year: Option<u16>,
    disposals: Vec<Disposal>,
    total_gain: String,
    total_loss: String,
    net_gain: String,
    total_proceeds: String,
    total_cost: String,
    exemption: String,
    taxable_gain: String,
    tax_liability: String,
}

/// Tax report with all calculated fields
#[derive(Serialize)]
struct TaxReport {
    tax_years: Vec<TaxYear>,
    holdings: Vec<serde_json::Value>,
}

/// Calculate tax report from transaction DSL and return JSON representation.
///
/// # Arguments
/// * `dsl` - Transaction DSL string
/// * `tax_year` - Optional tax year start (e.g., 2024 for 2024/25). If null, includes all years.
///
/// # Returns
/// JSON representation of tax report with exemption, taxable_gain, tax_liability, and disposal details
///
/// # Example
/// ```javascript
/// const dsl = `
///   BUY 2024-01-15 AAPL 10 @ 150.00 USD
///   SELL 2024-06-20 AAPL 5 @ 180.00 USD
/// `;
/// const report = JSON.parse(calculate_tax(dsl, 2024));
/// console.log(report.tax_years[0].total_gain);
/// console.log(report.tax_years[0].exemption);
/// console.log(report.tax_years[0].taxable_gain);
/// ```
#[wasm_bindgen]
pub fn calculate_tax(dsl: &str, tax_year: Option<i32>) -> Result<String, JsValue> {
    // Parse transactions
    let transactions = parser::parse_file(dsl).map_err(map_error)?;

    // Load bundled FX rates (embedded in WASM)
    let fx_cache = load_default_cache().map_err(map_error)?;

    // Load embedded config
    let config = Config::embedded().map_err(map_error)?;

    // Calculate tax report
    let report = calculator::calculate(&transactions, tax_year, Some(&fx_cache), &config)
        .map_err(map_error)?;

    // Enhance with exemption and tax calculations
    let tax_years: Vec<TaxYear> = report
        .tax_years
        .iter()
        .map(|ty| {
            let start_year = ty.period.start_year();
            let exemption = ty.exempt_amount;
            let gross_proceeds: Decimal = ty.disposals.iter().map(|d| d.gross_proceeds).sum();
            let total_cost: Decimal = ty
                .disposals
                .iter()
                .flat_map(|d| d.matches.iter())
                .map(|m| m.allowable_cost)
                .sum();
            let taxable = (ty.net_gain - exemption).max(Decimal::ZERO);

            // Tax liability calculation (simplified - set to zero for now)
            let tax_liability = Decimal::ZERO;

            TaxYear {
                period: Some(cgt_format::format_tax_year(start_year)),
                year: Some(start_year),
                disposals: ty.disposals.clone(),
                total_gain: ty.total_gain.to_string(),
                total_loss: ty.total_loss.to_string(),
                net_gain: ty.net_gain.to_string(),
                total_proceeds: gross_proceeds.to_string(),
                total_cost: total_cost.to_string(),
                exemption: exemption.to_string(),
                taxable_gain: taxable.to_string(),
                tax_liability: tax_liability.to_string(),
            }
        })
        .collect();

    let holdings: Result<Vec<serde_json::Value>, _> = report
        .holdings
        .iter()
        .map(|h| serde_json::to_value(h).map_err(map_error))
        .collect();

    let tax_report = TaxReport {
        tax_years,
        holdings: holdings?,
    };

    serde_json::to_string_pretty(&tax_report).map_err(map_error)
}

/// Validate transaction DSL and return validation result as JSON.
///
/// # Arguments
/// * `dsl` - Transaction DSL string
///
/// # Returns
/// JSON object with validation result including errors and warnings
///
/// # Example
/// ```javascript
/// const dsl = `BUY 2024-01-15 AAPL 10 @ 150.00 USD`;
/// const result = JSON.parse(validate_dsl(dsl));
/// if (!result.is_valid) {
///   console.error("Validation errors:", result.errors);
/// }
/// if (result.warnings.length > 0) {
///   console.warn("Warnings:", result.warnings);
/// }
/// ```
#[wasm_bindgen]
pub fn validate_dsl(dsl: &str) -> Result<String, JsValue> {
    let transactions = parser::parse_file(dsl).map_err(map_error)?;
    let result = validate(&transactions);
    serde_json::to_string_pretty(&result).map_err(map_error)
}
