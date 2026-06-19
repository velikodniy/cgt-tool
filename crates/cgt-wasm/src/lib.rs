use cgt::Config;
use cgt::money::load_default_cache;
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
    let transactions = cgt::dsl::parse(dsl).map_err(map_error)?;
    serde_json::to_string_pretty(&transactions).map_err(map_error)
}

/// Calculate tax report from transaction DSL and return JSON representation.
///
/// # Arguments
/// * `dsl` - Transaction DSL string
/// * `tax_year` - Optional tax year start (e.g., 2024 for 2024/25). If null, includes all years.
///
/// # Returns
/// JSON representation of the tax report with per-year gross_proceeds,
/// total_allowable_cost, exempt_amount, taxable_gain, and disposal details
///
/// # Example
/// ```javascript
/// const dsl = `
///   BUY 2024-01-15 AAPL 10 @ 150.00 USD
///   SELL 2024-06-20 AAPL 5 @ 180.00 USD
/// `;
/// const report = JSON.parse(calculate_tax(dsl, 2024));
/// console.log(report.tax_years[0].total_gain);
/// console.log(report.tax_years[0].exempt_amount);
/// console.log(report.tax_years[0].taxable_gain);
/// ```
#[wasm_bindgen]
pub fn calculate_tax(dsl: &str, tax_year: Option<i32>) -> Result<String, JsValue> {
    let transactions = cgt::dsl::parse(dsl).map_err(map_error)?;
    let fx = load_default_cache().map_err(map_error)?;
    let config = Config::embedded().map_err(map_error)?;
    let report = cgt::calculate(&transactions, tax_year, Some(&fx), &config).map_err(map_error)?;
    serde_json::to_string_pretty(&report).map_err(map_error)
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
    let transactions = cgt::dsl::parse(dsl).map_err(map_error)?;
    let result = cgt::validate(&transactions);
    serde_json::to_string_pretty(&result).map_err(map_error)
}
