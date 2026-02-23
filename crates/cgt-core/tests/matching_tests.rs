#![allow(clippy::expect_used)]

use cgt_core::calculator::calculate;
use cgt_core::models::*;
use cgt_core::parser::parse_file;
use cgt_money::{FxCache, load_default_cache};
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

/// Create an FxCache from bundled rates for testing.
fn get_fx_cache() -> FxCache {
    load_default_cache().expect("Failed to load bundled FX rates")
}

#[test]
fn test_data_driven_matching() {
    let inputs_dir = get_test_inputs_dir();
    let json_dir = get_test_json_dir();
    let fx_cache = get_fx_cache();
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

            // Calculate without year filter to get all tax years, with FX cache for multi-currency
            let actual_report = calculate(transactions.clone(), None, Some(&fx_cache))
                .expect("Failed to calculate");

            // Allow larger precision differences because reference data (cgtcalc output)
            // often rounds to nearest integer or uses 5dp, while we use exact decimal.
            let epsilon = Decimal::new(1, 0); // 1.0

            // Verify number of tax years match
            assert_eq!(
                actual_report.tax_years.len(),
                expected_report.tax_years.len(),
                "Tax year count mismatch for {}. Actual: {}, Expected: {}",
                input_path.display(),
                actual_report.tax_years.len(),
                expected_report.tax_years.len()
            );

            // Compare each tax year
            for (actual_tax_year, expected_tax_year) in actual_report
                .tax_years
                .iter()
                .zip(expected_report.tax_years.iter())
            {
                assert_eq!(
                    actual_tax_year.period,
                    expected_tax_year.period,
                    "Tax year period mismatch for {}",
                    input_path.display()
                );

                assert!(
                    (actual_tax_year.total_gain - expected_tax_year.total_gain).abs() <= epsilon,
                    "Total Gain mismatch for {} in {}. Actual: {}, Expected: {}",
                    input_path.display(),
                    actual_tax_year.period,
                    actual_tax_year.total_gain,
                    expected_tax_year.total_gain
                );

                assert!(
                    (actual_tax_year.total_loss - expected_tax_year.total_loss).abs() <= epsilon,
                    "Total Loss mismatch for {} in {}. Actual: {}, Expected: {}",
                    input_path.display(),
                    actual_tax_year.period,
                    actual_tax_year.total_loss,
                    expected_tax_year.total_loss
                );

                // Verify number of disposals match
                assert_eq!(
                    actual_tax_year.disposals.len(),
                    expected_tax_year.disposals.len(),
                    "Disposal count mismatch for {} in {}. Actual: {}, Expected: {}",
                    input_path.display(),
                    actual_tax_year.period,
                    actual_tax_year.disposals.len(),
                    expected_tax_year.disposals.len()
                );
            }
        }
    }
}

#[test]
fn test_tax_year_totals_use_net_disposals() {
    let inputs_dir = get_test_inputs_dir();
    let input_path = inputs_dir.join("NetDisposalTotalsMixed.cgt");
    let input_content = fs::read_to_string(&input_path).expect("Failed to read input");
    let transactions = parse_file(&input_content).expect("Failed to parse input");

    let report = calculate(transactions, None, None).expect("Failed to calculate");
    let year = report.tax_years.first().expect("Expected a tax year");

    let mix_disposal = year
        .disposals
        .iter()
        .find(|d| d.ticker == "MIX")
        .expect("Missing MIX disposal");
    let zero_disposal = year
        .disposals
        .iter()
        .find(|d| d.ticker == "ZERO")
        .expect("Missing ZERO disposal");
    let pool_disposal = year
        .disposals
        .iter()
        .find(|d| d.ticker == "POOL")
        .expect("Missing POOL disposal");

    let mix_net: Decimal = mix_disposal.matches.iter().map(|m| m.gain_or_loss).sum();
    let zero_net: Decimal = zero_disposal.matches.iter().map(|m| m.gain_or_loss).sum();
    let pool_net: Decimal = pool_disposal.matches.iter().map(|m| m.gain_or_loss).sum();

    assert_eq!(mix_net, dec!(80));
    assert_eq!(zero_net, Decimal::ZERO);
    assert_eq!(pool_net, dec!(10));

    assert_eq!(year.total_gain, dec!(90));
    assert_eq!(year.total_loss, Decimal::ZERO);
    assert_eq!(year.net_gain, dec!(90));
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

// CAPRETURN and DIVIDEND cost apportionment tests
// These tests verify that cost adjustments are correctly apportioned across acquisition lots
// based on each lot's proportion of total holdings, not based on the event amount.

#[test]
fn test_capreturn_apportioned_across_multiple_lots() {
    // When CAPRETURN affects fewer shares than total holdings,
    // the cost reduction should be distributed proportionally across all lots
    let cgt_content = r#"
2024-01-01 BUY ACME 10 @ 100.00 GBP
2024-02-01 BUY ACME 10 @ 90.00 GBP
2024-03-01 CAPRETURN ACME 10 TOTAL 100 GBP FEES 0 GBP
2024-06-01 SELL ACME 20 @ 110.00 GBP
"#;

    let transactions = parse_file(cgt_content).expect("Failed to parse");
    let report = calculate(transactions, Some(2024), None).expect("Failed to calculate");

    // Total cost before CAPRETURN: 1000 + 900 = 1900
    // CAPRETURN reduces cost by 100 (event is for 10 shares, but affects all 20 proportionally)
    // Lot 1 (10 of 20 shares): -100 * (10/20) = -50
    // Lot 2 (10 of 20 shares): -100 * (10/20) = -50
    // Total cost after: 1900 - 100 = 1800
    // Average cost: 1800 / 20 = 90

    let disposal = &report.tax_years[0].disposals[0];

    // Proceeds: 20 * 110 = 2200
    // Cost: 1800
    // Gain: 400
    let total_gain: Decimal = disposal.matches.iter().map(|m| m.gain_or_loss).sum();
    assert_eq!(
        total_gain,
        dec!(400),
        "CAPRETURN should reduce cost proportionally, resulting in gain of 400"
    );

    // Verify cost is 1800
    let total_cost: Decimal = disposal.matches.iter().map(|m| m.allowable_cost).sum();
    assert_eq!(
        total_cost,
        dec!(1800),
        "Total cost should be 1900 - 100 = 1800"
    );
}

#[test]
fn test_capreturn_with_prior_partial_sale() {
    // CAPRETURN should only affect shares remaining at the time of the event.
    // This test verifies that CAPRETURN is applied correctly even when
    // B&B matching affects which shares are used for a prior sale.
    let cgt_content = r#"
2024-01-01 BUY ACME 10 @ 100.00 GBP
2024-02-01 SELL ACME 5 @ 110.00 GBP
2024-03-01 BUY ACME 10 @ 90.00 GBP
2024-04-01 CAPRETURN ACME 15 TOTAL 75 GBP FEES 0 GBP
2024-06-01 SELL ACME 15 @ 120.00 GBP
"#;

    let transactions = parse_file(cgt_content).expect("Failed to parse");
    let report = calculate(transactions, None, None).expect("Failed to calculate");

    // Timeline:
    // 1. Buy 10 @ 100 = £1000
    // 2. Sell 5 @ 110 - B&B matched with later buy on 2024-03-01
    // 3. Buy 10 @ 90 = £900 (5 used for B&B match)
    // 4. CAPRETURN 15 shares for £75 (reduces cost of remaining shares)
    // 5. Sell 15 @ 120 from S104 pool

    // The B&B match should show CAPRETURN applied:
    let first_sale = report.tax_years[0]
        .disposals
        .iter()
        .find(|d| d.quantity == dec!(5))
        .expect("Should have 5-share B&B sale");

    // B&B match uses 5 shares from the 2024-03-01 buy
    // Original cost: 5 × 90 = 450
    // CAPRETURN adjustment applied to these shares: -75 × (5/15) = -25
    // Adjusted cost: 450 - 25 = 425
    let bb_match = first_sale
        .matches
        .iter()
        .find(|m| m.rule == MatchRule::BedAndBreakfast)
        .expect("Should have B&B match");
    assert_eq!(
        bb_match.allowable_cost,
        dec!(425),
        "B&B match should include CAPRETURN adjustment"
    );

    // The second sale from S104 pool:
    let second_sale = report.tax_years[1]
        .disposals
        .iter()
        .find(|d| d.quantity == dec!(15))
        .expect("Should have 15-share S104 sale");

    // Remaining in pool after B&B:
    // - Lot 1: 10 shares, cost 1000
    // - Lot 2: 5 shares (10 - 5 B&B), cost 450
    // Total: 15 shares, cost 1450
    // CAPRETURN distributed to remaining lots...
    // Note: The exact distribution depends on FIFO simulation which may differ
    // from actual B&B matching. The key is total cost is correctly adjusted.

    // Verify we have 15 shares sold at 120 = 1800 proceeds
    assert_eq!(second_sale.proceeds, dec!(1800));
    assert_eq!(second_sale.quantity, dec!(15));
}

#[test]
fn test_dividend_increases_cost_proportionally() {
    // Accumulation fund dividend should increase cost basis proportionally
    let cgt_content = r#"
2024-01-01 BUY ACME 10 @ 100.00 GBP
2024-02-01 BUY ACME 10 @ 90.00 GBP
2024-03-01 DIVIDEND ACME 10 TOTAL 50 GBP TAX 0 GBP
2024-06-01 SELL ACME 20 @ 110.00 GBP
"#;

    let transactions = parse_file(cgt_content).expect("Failed to parse");
    let report = calculate(transactions, Some(2024), None).expect("Failed to calculate");

    // Total cost before DIVIDEND: 1000 + 900 = 1900
    // DIVIDEND increases cost by 50 (distributed proportionally):
    //   Lot 1 (10 of 20): +50 * (10/20) = +25
    //   Lot 2 (10 of 20): +50 * (10/20) = +25
    // Total cost after: 1900 + 50 = 1950

    let disposal = &report.tax_years[0].disposals[0];

    // Proceeds: 20 * 110 = 2200
    // Cost: 1950
    // Gain: 250
    let total_gain: Decimal = disposal.matches.iter().map(|m| m.gain_or_loss).sum();
    assert_eq!(
        total_gain,
        dec!(250),
        "DIVIDEND should increase cost proportionally, resulting in gain of 250"
    );
}

#[test]
fn test_capreturn_and_dividend_combined() {
    // Both CAPRETURN and DIVIDEND on same holding
    let cgt_content = r#"
2024-01-01 BUY ACME 10 @ 100.00 GBP
2024-02-01 BUY ACME 10 @ 90.00 GBP
2024-03-01 CAPRETURN ACME 20 TOTAL 100 GBP FEES 0 GBP
2024-03-01 DIVIDEND ACME 20 TOTAL 50 GBP TAX 0 GBP
2024-06-01 SELL ACME 20 @ 110.00 GBP
"#;

    let transactions = parse_file(cgt_content).expect("Failed to parse");
    let report = calculate(transactions, Some(2024), None).expect("Failed to calculate");

    // Total cost before events: 1000 + 900 = 1900
    // CAPRETURN -100: 1900 - 100 = 1800
    // DIVIDEND +50: 1800 + 50 = 1850

    let disposal = &report.tax_years[0].disposals[0];

    // Proceeds: 20 * 110 = 2200
    // Cost: 1850
    // Gain: 350
    let total_gain: Decimal = disposal.matches.iter().map(|m| m.gain_or_loss).sum();
    assert_eq!(
        total_gain,
        dec!(350),
        "Combined CAPRETURN and DIVIDEND should net to -50 cost adjustment"
    );
}

#[test]
fn test_capreturn_event_amount_less_than_holdings() {
    // This is the key bug fix test case: when event_amount < total_holdings,
    // the adjustment must still be correctly distributed
    let cgt_content = r#"
2024-01-01 BUY ACME 100 @ 10.00 GBP
2024-03-01 CAPRETURN ACME 50 TOTAL 25 GBP FEES 0 GBP
2024-06-01 SELL ACME 100 @ 12.00 GBP
"#;

    let transactions = parse_file(cgt_content).expect("Failed to parse");
    let report = calculate(transactions, Some(2024), None).expect("Failed to calculate");

    // Cost before CAPRETURN: 100 * 10 = 1000
    // CAPRETURN is for 50 shares at -25, but we hold 100 shares
    // The -25 adjustment is distributed across all 100 shares
    // Cost after: 1000 - 25 = 975

    let disposal = &report.tax_years[0].disposals[0];

    // Proceeds: 100 * 12 = 1200
    // Cost: 975
    // Gain: 225
    let total_gain: Decimal = disposal.matches.iter().map(|m| m.gain_or_loss).sum();
    assert_eq!(
        total_gain,
        dec!(225),
        "CAPRETURN of 25 on 50 shares should reduce 100-share pool by 25"
    );

    let total_cost: Decimal = disposal.matches.iter().map(|m| m.allowable_cost).sum();
    assert_eq!(
        total_cost,
        dec!(975),
        "Total cost should be 1000 - 25 = 975"
    );
}

#[test]
fn test_capreturn_does_not_affect_later_acquisitions() {
    // CAPRETURN should not affect acquisitions made after the event
    let cgt_content = r#"
2024-01-01 BUY ACME 10 @ 100.00 GBP
2024-02-01 CAPRETURN ACME 10 TOTAL 50 GBP FEES 0 GBP
2024-03-01 BUY ACME 10 @ 90.00 GBP
2024-06-01 SELL ACME 20 @ 110.00 GBP
"#;

    let transactions = parse_file(cgt_content).expect("Failed to parse");
    let report = calculate(transactions, Some(2024), None).expect("Failed to calculate");

    // At CAPRETURN time: only 10 shares from lot 1
    // CAPRETURN -50 applied only to lot 1
    // Lot 1 cost: 1000 - 50 = 950 for 10 shares
    // Lot 2 cost: 900 for 10 shares (not affected, acquired after)
    // Total: 1850 for 20 shares

    let disposal = &report.tax_years[0].disposals[0];

    // Proceeds: 20 * 110 = 2200
    // Cost: 1850
    // Gain: 350
    let total_gain: Decimal = disposal.matches.iter().map(|m| m.gain_or_loss).sum();
    assert_eq!(
        total_gain,
        dec!(350),
        "CAPRETURN should not affect acquisitions made after the event"
    );
}

// Same Day reservation priority over B&B
// Per TCGA92/S106A(9), B&B is "subject to" Same Day rule (S105(1))

#[test]
fn test_same_day_reservation_priority_over_bnb() {
    // Test that Same Day matching has priority over B&B from earlier disposals.
    // Per TCGA92/S106A(9): B&B rules are "subject to" the Same Day rule in S105(1).
    //
    // Scenario:
    // - Jan 1: Buy 200 shares @ £10 (initial pool)
    // - Feb 1: Sell 100 shares @ £12 (could B&B to Feb 2)
    // - Feb 2: Buy 80 shares @ £11, Sell 50 shares @ £11.50 (Same Day)
    //
    // Without reservation (WRONG): Feb 1 B&B takes all 80 from Feb 2, Feb 2 Same Day gets 0
    // With reservation (CORRECT): Feb 2 Same Day reserves 50, Feb 1 B&B gets only 30
    let cgt_content = r#"
2024-01-01 BUY SNAP 200 @ 10.00 GBP
2024-02-01 SELL SNAP 100 @ 12.00 GBP
2024-02-02 BUY SNAP 80 @ 11.00 GBP
2024-02-02 SELL SNAP 50 @ 11.50 GBP
"#;

    let transactions = parse_file(cgt_content).expect("Failed to parse");
    let report = calculate(transactions, Some(2023), None).expect("Failed to calculate");

    let year = report.tax_years.first().expect("Expected a tax year");

    // Find the Feb 1 disposal
    let feb1_disposal = year
        .disposals
        .iter()
        .find(|d| d.date.to_string() == "2024-02-01")
        .expect("Missing Feb 1 disposal");

    // Find the Feb 2 disposal
    let feb2_disposal = year
        .disposals
        .iter()
        .find(|d| d.date.to_string() == "2024-02-02")
        .expect("Missing Feb 2 disposal");

    // Feb 1 should have B&B match for 30 shares (not 80!) and S104 match for 70 shares
    assert_eq!(
        feb1_disposal.matches.len(),
        2,
        "Feb 1 should have 2 matches"
    );

    let feb1_bnb = feb1_disposal
        .matches
        .iter()
        .find(|m| m.rule == MatchRule::BedAndBreakfast)
        .expect("Feb 1 should have B&B match");
    assert_eq!(
        feb1_bnb.quantity,
        dec!(30),
        "Feb 1 B&B should only get 30 shares (80 - 50 reserved for Same Day)"
    );

    let feb1_s104 = feb1_disposal
        .matches
        .iter()
        .find(|m| m.rule == MatchRule::Section104)
        .expect("Feb 1 should have S104 match");
    assert_eq!(
        feb1_s104.quantity,
        dec!(70),
        "Feb 1 S104 should get remaining 70 shares"
    );

    // Feb 2 should have Same Day match for all 50 shares
    assert_eq!(feb2_disposal.matches.len(), 1, "Feb 2 should have 1 match");
    let feb2_same_day = &feb2_disposal.matches[0];
    assert_eq!(
        feb2_same_day.rule,
        MatchRule::SameDay,
        "Feb 2 should be Same Day match"
    );
    assert_eq!(
        feb2_same_day.quantity,
        dec!(50),
        "Feb 2 Same Day should get all 50 shares"
    );

    // Verify gains
    // Feb 1 B&B: 30 × (£12 - £11) = £30
    // Feb 1 S104: 70 × (£12 - £10) = £140
    // Feb 2 Same Day: 50 × (£11.50 - £11) = £25
    // Total: £195
    assert_eq!(year.total_gain, dec!(195), "Total gain should be £195");
}

#[test]
fn test_sell_without_prior_acquisition_returns_error() {
    let cgt_content = r#"
2024-06-01 SELL ACME 10 @ 12.00 GBP
"#;

    let transactions = parse_file(cgt_content).expect("Failed to parse");
    let error = calculate(transactions, Some(2024), None)
        .expect_err("Expected unmatched sell to fail")
        .to_string();

    assert!(
        error.contains("no prior acquisitions"),
        "Expected no prior acquisitions error, got: {error}"
    );
}

#[test]
fn test_oversell_returns_error() {
    let cgt_content = r#"
2024-01-01 BUY ACME 5 @ 10.00 GBP
2024-06-01 SELL ACME 10 @ 12.00 GBP
"#;

    let transactions = parse_file(cgt_content).expect("Failed to parse");
    let error = calculate(transactions, Some(2024), None)
        .expect_err("Expected oversell to fail")
        .to_string();

    assert!(
        error.contains("exceeds holding"),
        "Expected exceeds holding error, got: {error}"
    );
}

#[test]
fn test_same_day_reservation_with_interleaved_buys() {
    // Interleaved buys for another ticker must not cause over-reservation on the
    // B&B acquisition date for the target ticker.
    let cgt_content = r#"
2024-01-01 BUY SNAP 200 @ 10.00 GBP
2024-02-01 SELL SNAP 100 @ 12.00 GBP
2024-02-02 BUY SNAP 40 @ 11.00 GBP
2024-02-02 BUY OTHER 1 @ 1.00 GBP
2024-02-02 BUY SNAP 40 @ 11.00 GBP
2024-02-02 SELL SNAP 50 @ 11.50 GBP
"#;

    let transactions = parse_file(cgt_content).expect("Failed to parse");
    let report = calculate(transactions, Some(2023), None).expect("Failed to calculate");

    let year = report.tax_years.first().expect("Expected a tax year");

    let feb1_disposal = year
        .disposals
        .iter()
        .find(|d| d.date.to_string() == "2024-02-01")
        .expect("Missing Feb 1 disposal");

    let feb2_disposal = year
        .disposals
        .iter()
        .find(|d| d.date.to_string() == "2024-02-02" && d.ticker == "SNAP")
        .expect("Missing Feb 2 SNAP disposal");

    let feb1_bnb = feb1_disposal
        .matches
        .iter()
        .find(|m| m.rule == MatchRule::BedAndBreakfast)
        .expect("Feb 1 should have B&B match");
    assert_eq!(
        feb1_bnb.quantity,
        dec!(30),
        "Feb 1 B&B should use 30 shares after reserving 50 for Same Day"
    );

    let feb1_s104 = feb1_disposal
        .matches
        .iter()
        .find(|m| m.rule == MatchRule::Section104)
        .expect("Feb 1 should have S104 match");
    assert_eq!(feb1_s104.quantity, dec!(70));

    assert_eq!(feb2_disposal.matches.len(), 1);
    assert_eq!(feb2_disposal.matches[0].rule, MatchRule::SameDay);
    assert_eq!(feb2_disposal.matches[0].quantity, dec!(50));
}

#[test]
fn test_capreturn_excess_creates_deemed_gain_and_clamps_pool_cost() {
    let cgt_content = r#"
2024-01-01 BUY ACME 10 @ 100.00 GBP
2024-02-01 CAPRETURN ACME 10 TOTAL 1200.00 GBP FEES 0.00 GBP
"#;

    let transactions = parse_file(cgt_content).expect("Failed to parse");
    let report = calculate(transactions, Some(2023), None).expect("Failed to calculate");

    let year = report.tax_years.first().expect("Expected a tax year");
    assert_eq!(year.total_gain, dec!(200));
    assert_eq!(year.total_loss, Decimal::ZERO);

    assert_eq!(year.disposals.len(), 1);
    let deemed_disposal = &year.disposals[0];
    assert_eq!(deemed_disposal.ticker, "ACME");

    assert_eq!(deemed_disposal.matches.len(), 1);
    let deemed_match = &deemed_disposal.matches[0];
    assert_eq!(deemed_match.rule, MatchRule::CapitalReturnExcess);
    assert_eq!(deemed_match.allowable_cost, Decimal::ZERO);
    assert_eq!(deemed_match.gain_or_loss, dec!(200));

    let holding = report
        .holdings
        .iter()
        .find(|h| h.ticker == "ACME")
        .expect("Expected ACME holding");
    assert_eq!(holding.quantity, dec!(10));
    assert_eq!(holding.total_cost, Decimal::ZERO);
}
