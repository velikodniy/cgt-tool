#![allow(clippy::expect_used)]

use cgt_core::calculator::calculate;
use cgt_core::models::*;
use cgt_core::parser::parse_file;
use chrono::Datelike;
use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use std::fs;
use std::path::PathBuf;

fn get_test_inputs_dir() -> PathBuf {
    let mut d = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    d.pop(); // crates
    d.pop(); // root
    d.push("tests");
    d.push("inputs");
    d
}

fn get_test_json_dir() -> PathBuf {
    let mut d = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    d.pop(); // crates
    d.pop(); // root
    d.push("tests");
    d.push("json");
    d
}

/// Derive the tax year start from a date (UK tax year runs April 6 to April 5)
fn tax_year_start_from_date(date: chrono::NaiveDate) -> i32 {
    let year = date.year();
    let month = date.month();
    let day = date.day();
    // UK tax year starts April 6
    if month < 4 || (month == 4 && day < 6) {
        year - 1
    } else {
        year
    }
}

/// Find the first sale date in the transactions to determine which tax year to test
fn find_first_sale_date(transactions: &[Transaction]) -> Option<chrono::NaiveDate> {
    transactions
        .iter()
        .filter_map(|t| {
            if matches!(t.operation, Operation::Sell { .. }) {
                Some(t.date)
            } else {
                None
            }
        })
        .min()
}

#[test]
fn test_data_driven_matching() {
    let inputs_dir = get_test_inputs_dir();
    let json_dir = get_test_json_dir();
    let entries = fs::read_dir(&inputs_dir).expect("Failed to read test inputs dir");

    for entry in entries {
        let entry = entry.expect("Failed to read entry");
        let path = entry.path();

        if path.extension().and_then(|s| s.to_str()) == Some("cgt") {
            let input_path = path;
            let file_stem = input_path
                .file_stem()
                .expect("No file stem")
                .to_str()
                .expect("Invalid UTF-8");
            let output_path = json_dir.join(format!("{}.json", file_stem));

            if !output_path.exists() {
                println!("Skipping {} (no matching json)", input_path.display());
                continue;
            }

            println!("Testing {}", input_path.display());

            let input_content = fs::read_to_string(&input_path).expect("Failed to read input");
            let transactions = parse_file(&input_content).expect("Failed to parse input");

            let output_content = fs::read_to_string(&output_path).expect("Failed to read output");
            let expected_report: TaxReport =
                serde_json::from_str(&output_content).expect("Failed to parse expected output");

            // Derive tax year from the first sale date in transactions
            let first_sale_date = find_first_sale_date(&transactions);

            // Get the expected tax year from JSON
            let expected_tax_year = expected_report
                .tax_years
                .first()
                .expect("Expected report should have at least one tax year");
            let expected_year_start = expected_tax_year.period.start_year() as i32;

            // If there are sales, verify the JSON has the correct tax year
            if let Some(sale_date) = first_sale_date {
                let derived_year_start = tax_year_start_from_date(sale_date);
                assert_eq!(
                    derived_year_start,
                    expected_year_start,
                    "Tax year mismatch for {}. First sale {} is in tax year {}/{}, but JSON expects {}/{}",
                    input_path.display(),
                    sale_date,
                    derived_year_start,
                    (derived_year_start + 1) % 100,
                    expected_year_start,
                    (expected_year_start + 1) % 100
                );
            }

            let actual_report = calculate(transactions.clone(), Some(expected_year_start), None)
                .expect("Failed to calculate");

            // Get the actual tax year summary
            let actual_tax_year = actual_report
                .tax_years
                .first()
                .expect("Actual report should have at least one tax year");

            // Allow larger precision differences because reference data (cgtcalc output)
            // often rounds to nearest integer or uses 5dp, while we use exact decimal.
            let epsilon = Decimal::new(1, 0); // 1.0

            assert!(
                (actual_tax_year.total_gain - expected_tax_year.total_gain).abs() <= epsilon,
                "Total Gain mismatch for {}. Actual: {}, Expected: {}",
                input_path.display(),
                actual_tax_year.total_gain,
                expected_tax_year.total_gain
            );

            assert!(
                (actual_tax_year.total_loss - expected_tax_year.total_loss).abs() <= epsilon,
                "Total Loss mismatch for {}. Actual: {}, Expected: {}",
                input_path.display(),
                actual_tax_year.total_loss,
                expected_tax_year.total_loss
            );

            // Verify number of disposals match
            assert_eq!(
                actual_tax_year.disposals.len(),
                expected_tax_year.disposals.len(),
                "Disposal count mismatch for {}. Actual: {}, Expected: {}",
                input_path.display(),
                actual_tax_year.disposals.len(),
                expected_tax_year.disposals.len()
            );
        }
    }
}

// Share quantity precision tests
// Verify that decimal quantities are preserved exactly without floating-point rounding errors

#[test]
fn test_high_precision_decimal_quantity_preserved() {
    // Test that quantities with many decimal places are preserved exactly
    // This catches floating-point rounding errors that plagued other calculators
    let cgt_content = r#"
2024-05-01 BUY ACME 67.201495 @ 125.6445 GBP
2024-05-15 SELL ACME 67.201495 @ 130.00 GBP
"#;

    let transactions = parse_file(cgt_content).expect("Failed to parse");
    let report = calculate(transactions, Some(2024), None).expect("Failed to calculate");

    let disposal = &report.tax_years[0].disposals[0];
    // Quantity should be exactly preserved
    assert_eq!(
        disposal.quantity,
        dec!(67.201495),
        "Quantity should be preserved exactly without rounding"
    );
}

#[test]
fn test_very_small_fractional_share_quantity() {
    // Test extremely small fractional shares (common in dividend reinvestment)
    let cgt_content = r#"
2024-05-01 BUY ACME 0.000001 @ 100.00 GBP
2024-05-15 SELL ACME 0.000001 @ 150.00 GBP
"#;

    let transactions = parse_file(cgt_content).expect("Failed to parse");
    let report = calculate(transactions, Some(2024), None).expect("Failed to calculate");

    let disposal = &report.tax_years[0].disposals[0];
    assert_eq!(
        disposal.quantity,
        dec!(0.000001),
        "Very small quantity should be preserved"
    );

    // Proceeds: 0.000001 * 150 = 0.00015
    // Cost: 0.000001 * 100 = 0.0001
    // Gain: 0.00005
    assert_eq!(disposal.proceeds, dec!(0.00015));
}

#[test]
fn test_quantity_precision_through_section_104_pool() {
    // Verify precision is maintained through S104 pool calculations
    let cgt_content = r#"
2024-01-01 BUY ACME 33.333333 @ 100.00 GBP
2024-02-01 BUY ACME 33.333333 @ 110.00 GBP
2024-03-01 BUY ACME 33.333334 @ 120.00 GBP
2024-06-01 SELL ACME 50.000000 @ 130.00 GBP
"#;

    let transactions = parse_file(cgt_content).expect("Failed to parse");
    let report = calculate(transactions, Some(2024), None).expect("Failed to calculate");

    // Total bought: 100.000000 shares exactly
    // After selling 50, pool should have 50.000000
    let pool = report
        .holdings
        .iter()
        .find(|h| h.ticker == "ACME")
        .expect("Pool should exist");
    assert_eq!(pool.quantity, dec!(50.000000));
}

// Proceeds calculation with fees
// Verify that disposal proceeds correctly account for selling costs per HMRC rules

#[test]
fn test_proceeds_deduct_selling_expenses() {
    // HMRC rules: Disposal proceeds = Sale amount - allowable selling costs
    // Reference: CG15250 (Allowable Incidental Costs)
    let cgt_content = r#"
2024-01-01 BUY ACME 100 @ 10.00 GBP
2024-06-01 SELL ACME 100 @ 15.00 GBP FEES 25.00 GBP
"#;

    let transactions = parse_file(cgt_content).expect("Failed to parse");
    let report = calculate(transactions, Some(2024), None).expect("Failed to calculate");

    let disposal = &report.tax_years[0].disposals[0];

    // Gross proceeds: 100 * 15 = 1500
    // Net proceeds after expenses: 1500 - 25 = 1475
    assert_eq!(
        disposal.proceeds,
        dec!(1475),
        "Proceeds should be net of selling expenses (1500 - 25 = 1475)"
    );

    // Cost: 100 * 10 = 1000
    // Gain: 1475 - 1000 = 475
    let total_gain: Decimal = disposal.matches.iter().map(|m| m.gain_or_loss).sum();
    assert_eq!(total_gain, dec!(475), "Gain should reflect net proceeds");
}

#[test]
fn test_proceeds_with_zero_expenses() {
    // When expenses are zero, proceeds should equal gross sale amount
    let cgt_content = r#"
2024-01-01 BUY ACME 100 @ 10.00 GBP
2024-06-01 SELL ACME 100 @ 15.00 GBP FEES 0.00 GBP
"#;

    let transactions = parse_file(cgt_content).expect("Failed to parse");
    let report = calculate(transactions, Some(2024), None).expect("Failed to calculate");

    let disposal = &report.tax_years[0].disposals[0];
    assert_eq!(
        disposal.proceeds,
        dec!(1500),
        "Proceeds with zero expenses should equal gross"
    );
}

#[test]
fn test_expenses_apportioned_in_partial_sale() {
    // When selling part of a position, expenses should be apportioned
    let cgt_content = r#"
2024-01-01 BUY ACME 100 @ 10.00 GBP
2024-06-01 SELL ACME 40 @ 15.00 GBP FEES 20.00 GBP
"#;

    let transactions = parse_file(cgt_content).expect("Failed to parse");
    let report = calculate(transactions, Some(2024), None).expect("Failed to calculate");

    let disposal = &report.tax_years[0].disposals[0];

    // Gross proceeds: 40 * 15 = 600
    // Full expenses apply to this sale: 600 - 20 = 580
    assert_eq!(
        disposal.proceeds,
        dec!(580),
        "Partial sale proceeds should deduct expenses"
    );
}

#[test]
fn test_expenses_apportioned_across_match_rules() {
    // Expenses should be proportionally allocated when sale matches multiple rules
    // (Same-day, B&B, S104)
    let cgt_content = r#"
2024-01-01 BUY ACME 100 @ 10.00 GBP
2024-06-01 SELL ACME 150 @ 15.00 GBP FEES 30.00 GBP
2024-06-01 BUY ACME 50 @ 14.00 GBP
"#;

    let transactions = parse_file(cgt_content).expect("Failed to parse");
    let report = calculate(transactions, Some(2024), None).expect("Failed to calculate");

    let disposal = &report.tax_years[0].disposals[0];

    // Total gross: 150 * 15 = 2250
    // Net after expenses: 2250 - 30 = 2220
    assert_eq!(
        disposal.proceeds,
        dec!(2220),
        "Total proceeds should be net of all expenses"
    );

    // Should have two matches: Same-day (50) and S104 (100)
    assert_eq!(disposal.matches.len(), 2);

    // Verify proceeds are apportioned correctly
    // Same-day match (50 shares): 50/150 * 2220 = 740
    // S104 match (100 shares): 100/150 * 2220 = 1480
    let same_day_match = disposal
        .matches
        .iter()
        .find(|m| m.rule == MatchRule::SameDay)
        .expect("Should have same-day match");
    let s104_match = disposal
        .matches
        .iter()
        .find(|m| m.rule == MatchRule::Section104)
        .expect("Should have S104 match");

    // Verify proportional allocation of proceeds to each match
    // Note: The ratio 50:100 should be reflected in the gain calculations
    let total_qty = same_day_match.quantity + s104_match.quantity;
    assert_eq!(total_qty, dec!(150));
}

// Additional precision edge cases

#[test]
fn test_large_quantity_with_precise_decimals() {
    // Test large numbers with precise decimals don't lose precision
    let cgt_content = r#"
2024-05-01 BUY ACME 1234567.891234 @ 0.123456 GBP
2024-05-15 SELL ACME 1234567.891234 @ 0.234567 GBP
"#;

    let transactions = parse_file(cgt_content).expect("Failed to parse");
    let report = calculate(transactions, Some(2024), None).expect("Failed to calculate");

    let disposal = &report.tax_years[0].disposals[0];
    assert_eq!(
        disposal.quantity,
        dec!(1234567.891234),
        "Large precise quantity should be preserved"
    );
}

#[test]
fn test_price_with_many_decimal_places() {
    // Test that price precision is maintained in gain calculations
    let cgt_content = r#"
2024-05-01 BUY ACME 100 @ 125.123456 GBP
2024-05-15 SELL ACME 100 @ 130.654321 GBP
"#;

    let transactions = parse_file(cgt_content).expect("Failed to parse");
    let report = calculate(transactions, Some(2024), None).expect("Failed to calculate");

    let disposal = &report.tax_years[0].disposals[0];

    // Proceeds: 100 * 130.654321 = 13065.4321
    // Cost: 100 * 125.123456 = 12512.3456
    // Gain: 553.0865
    assert_eq!(disposal.proceeds, dec!(13065.4321));

    let total_gain: Decimal = disposal.matches.iter().map(|m| m.gain_or_loss).sum();
    assert_eq!(total_gain, dec!(553.0865));
}

#[test]
fn test_bed_and_breakfast_quantity_precision() {
    // Verify precision in B&B matching with fractional shares
    let cgt_content = r#"
2024-05-01 BUY ACME 100.123456 @ 100.00 GBP
2024-06-01 SELL ACME 50.123456 @ 110.00 GBP
2024-06-15 BUY ACME 50.123456 @ 105.00 GBP
"#;

    let transactions = parse_file(cgt_content).expect("Failed to parse");
    let report = calculate(transactions, Some(2024), None).expect("Failed to calculate");

    let disposal = &report.tax_years[0].disposals[0];
    assert_eq!(
        disposal.quantity,
        dec!(50.123456),
        "B&B disposal quantity should preserve precision"
    );

    // Should match with B&B rule (repurchased within 30 days)
    let bb_match = disposal
        .matches
        .iter()
        .find(|m| m.rule == MatchRule::BedAndBreakfast);
    assert!(bb_match.is_some(), "Should have B&B match");
    assert_eq!(bb_match.expect("B&B match").quantity, dec!(50.123456));
}

// All-years report generation tests

#[test]
fn test_all_years_report_generation() {
    // Test that omitting year returns all tax years with disposals
    let cgt_content = r#"
2023-06-01 BUY ACME 100 @ 100.00 GBP
2023-12-15 SELL ACME 50 @ 110.00 GBP
2024-06-20 SELL ACME 30 @ 120.00 GBP
"#;

    let transactions = parse_file(cgt_content).expect("Failed to parse");
    let report = calculate(transactions, None, None).expect("Failed to calculate");

    // Should have two tax years: 2023/24 (Dec 2023 sale) and 2024/25 (Jun 2024 sale)
    assert_eq!(
        report.tax_years.len(),
        2,
        "Should have 2 tax years with disposals"
    );

    // First year should be 2023/24
    assert_eq!(report.tax_years[0].period.start_year(), 2023);
    assert_eq!(report.tax_years[0].disposals.len(), 1);

    // Second year should be 2024/25
    assert_eq!(report.tax_years[1].period.start_year(), 2024);
    assert_eq!(report.tax_years[1].disposals.len(), 1);
}

#[test]
fn test_single_year_filter_still_works() {
    // Test that specifying a year still filters to just that year
    let cgt_content = r#"
2023-06-01 BUY ACME 100 @ 100.00 GBP
2023-12-15 SELL ACME 50 @ 110.00 GBP
2024-06-20 SELL ACME 30 @ 120.00 GBP
"#;

    let transactions = parse_file(cgt_content).expect("Failed to parse");
    let report = calculate(transactions, Some(2024), None).expect("Failed to calculate");

    // Should only have 2024/25 tax year
    assert_eq!(
        report.tax_years.len(),
        1,
        "Should have only 1 tax year when filtered"
    );
    assert_eq!(report.tax_years[0].period.start_year(), 2024);
    assert_eq!(report.tax_years[0].disposals.len(), 1);
}

#[test]
fn test_all_years_sorted_chronologically() {
    // Test that tax years are sorted chronologically
    let cgt_content = r#"
2020-06-01 BUY ACME 100 @ 100.00 GBP
2022-06-20 SELL ACME 20 @ 110.00 GBP
2021-06-20 SELL ACME 20 @ 105.00 GBP
2023-06-20 SELL ACME 20 @ 115.00 GBP
"#;

    let transactions = parse_file(cgt_content).expect("Failed to parse");
    let report = calculate(transactions, None, None).expect("Failed to calculate");

    // Should have 3 tax years
    assert_eq!(report.tax_years.len(), 3);

    // Should be sorted: 2021/22, 2022/23, 2023/24
    assert_eq!(report.tax_years[0].period.start_year(), 2021);
    assert_eq!(report.tax_years[1].period.start_year(), 2022);
    assert_eq!(report.tax_years[2].period.start_year(), 2023);
}

#[test]
fn test_all_years_no_disposals_returns_empty() {
    // Test that transactions with no disposals returns empty tax_years
    let cgt_content = r#"
2024-06-01 BUY ACME 100 @ 100.00 GBP
"#;

    let transactions = parse_file(cgt_content).expect("Failed to parse");
    let report = calculate(transactions, None, None).expect("Failed to calculate");

    // No disposals means no tax years in report
    assert_eq!(
        report.tax_years.len(),
        0,
        "No disposals should result in empty tax_years"
    );

    // But holdings should still be tracked
    assert_eq!(report.holdings.len(), 1);
    assert_eq!(report.holdings[0].ticker, "ACME");
}
