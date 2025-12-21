#![allow(clippy::panic)]

use cgt_wasm::{calculate_tax, parse_transactions, validate_dsl};
use wasm_bindgen_test::*;

#[wasm_bindgen_test]
fn test_parse_simple_transaction() {
    let dsl = "2024-01-15 BUY AAPL 10 @ 150.00 USD";
    let result = parse_transactions(dsl);

    match result {
        Ok(json) => {
            assert!(json.contains("AAPL"), "Expected AAPL in output");
            assert!(json.contains("150.00"), "Expected price in output");
        }
        Err(e) => panic!("Failed to parse: {:?}", e),
    }
}

#[wasm_bindgen_test]
fn test_parse_multiple_transactions() {
    let dsl = r#"
2024-01-15 BUY AAPL 10 @ 150.00 USD
2024-06-20 SELL AAPL 5 @ 180.00 USD
    "#;

    let result = parse_transactions(dsl);
    match result {
        Ok(json) => {
            assert!(json.contains("AAPL"), "Expected AAPL in output");
        }
        Err(e) => panic!("Failed to parse multiple transactions: {:?}", e),
    }
}

#[wasm_bindgen_test]
fn test_parse_invalid_dsl() {
    let dsl = "INVALID DSL STRING";
    let result = parse_transactions(dsl);

    assert!(result.is_err(), "Expected parse to fail for invalid DSL");
}

#[wasm_bindgen_test]
fn test_calculate_simple_gain() {
    let dsl = r#"
2024-01-15 BUY AAPL 10 @ 100.00 USD
2024-06-20 SELL AAPL 10 @ 150.00 USD
    "#;

    let result = calculate_tax(dsl, Some(2024));
    match result {
        Ok(json) => {
            // Verify JSON structure contains expected fields
            assert!(json.contains("tax_years"), "Expected tax_years in report");
            assert!(json.contains("holdings"), "Expected holdings in report");
        }
        Err(e) => panic!("Failed to calculate: {:?}", e),
    }
}

#[wasm_bindgen_test]
fn test_calculate_all_years() {
    let dsl = r#"
2023-01-15 BUY AAPL 10 @ 100.00 USD
2023-06-20 SELL AAPL 5 @ 150.00 USD
2024-06-20 SELL AAPL 5 @ 180.00 USD
    "#;

    let result = calculate_tax(dsl, None);
    match result {
        Ok(json) => {
            assert!(json.contains("tax_years"), "Expected tax_years in report");
        }
        Err(e) => panic!("Failed to calculate all years: {:?}", e),
    }
}

#[wasm_bindgen_test]
fn test_calculate_with_fx_conversion() {
    // This test verifies that bundled FX rates are embedded and working
    let dsl = r#"
2024-01-15 BUY AAPL 10 @ 150.00 USD
2024-06-20 SELL AAPL 5 @ 180.00 USD
    "#;

    let result = calculate_tax(dsl, Some(2024));
    assert!(
        result.is_ok(),
        "FX conversion should work with bundled rates: {:?}",
        result.err()
    );
}

#[wasm_bindgen_test]
fn test_validate_valid_dsl() {
    let dsl = "2024-01-15 BUY AAPL 10 @ 150.00 USD";

    let result = validate_dsl(dsl);
    match result {
        Ok(json) => {
            assert!(json.contains("is_valid"), "Expected is_valid field");
            assert!(json.contains("errors"), "Expected errors field");
            assert!(json.contains("warnings"), "Expected warnings field");
        }
        Err(e) => panic!("Failed to validate: {:?}", e),
    }
}

#[wasm_bindgen_test]
fn test_validate_zero_quantity() {
    let dsl = "2024-01-15 BUY AAPL 0 @ 150.00 USD";

    let result = validate_dsl(dsl);
    match result {
        Ok(json) => {
            // The validation should report an error about zero quantity
            assert!(
                json.contains("\"is_valid\":false") || json.contains("\"is_valid\": false"),
                "Should be invalid due to zero quantity"
            );
        }
        Err(e) => panic!("Validation should succeed even with errors in DSL: {:?}", e),
    }
}

#[wasm_bindgen_test]
fn test_parse_negative_quantity_error() {
    // Negative quantities are a parse error, not a validation error
    let dsl = "2024-01-15 BUY AAPL -10 @ 150.00 USD";

    let result = parse_transactions(dsl);
    assert!(result.is_err(), "Should fail to parse negative quantity");
}

#[wasm_bindgen_test]
fn test_validate_sell_before_buy_warning() {
    let dsl = r#"
2024-01-15 SELL AAPL 5 @ 150.00 USD
2024-02-20 BUY AAPL 10 @ 100.00 USD
    "#;

    let result = validate_dsl(dsl);
    match result {
        Ok(json) => {
            // Should have warnings but be valid
            assert!(json.contains("warnings"), "Expected warnings field");
        }
        Err(e) => panic!("Validation should succeed: {:?}", e),
    }
}

#[wasm_bindgen_test]
fn test_calculate_includes_exemption_and_taxable_gain() {
    let dsl = r#"
2024-01-15 BUY AAPL 10 @ 100.00 USD
2024-06-20 SELL AAPL 10 @ 150.00 USD
    "#;

    let result = calculate_tax(dsl, Some(2024));
    match result {
        Ok(json) => {
            // Verify enhanced fields are present
            assert!(json.contains("exemption"), "Expected exemption field");
            assert!(json.contains("taxable_gain"), "Expected taxable_gain field");
            assert!(
                json.contains("total_proceeds"),
                "Expected total_proceeds field"
            );
            assert!(json.contains("total_cost"), "Expected total_cost field");
            assert!(
                json.contains("tax_liability"),
                "Expected tax_liability field"
            );
        }
        Err(e) => panic!("Failed to calculate: {:?}", e),
    }
}
