#![allow(clippy::expect_used)]

use cgt_core::calculator::calculate;
use cgt_core::models::*;
use cgt_core::parser::parse_file;
use rust_decimal::Decimal;
use std::fs;
use std::path::PathBuf;

fn get_test_data_dir() -> PathBuf {
    let mut d = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    d.pop(); // crates
    d.pop(); // root
    d.push("tests");
    d.push("data");
    d
}

#[test]
fn test_data_driven_matching() {
    let data_dir = get_test_data_dir();
    let entries = fs::read_dir(&data_dir).expect("Failed to read test data dir");

    for entry in entries {
        let entry = entry.expect("Failed to read entry");
        let path = entry.path();

        if path.extension().and_then(|s| s.to_str()) == Some("cgt") {
            let input_path = path;
            let output_path = input_path.with_extension("json");

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

            // Get the tax year from the first tax year summary
            let first_tax_year = expected_report
                .tax_years
                .first()
                .expect("Expected report should have at least one tax year");
            let year_start = first_tax_year.period.start_year() as i32;

            let actual_report = calculate(transactions, year_start).expect("Failed to calculate");

            // Get the actual tax year summary
            let actual_tax_year = actual_report
                .tax_years
                .first()
                .expect("Actual report should have at least one tax year");

            // Allow larger precision differences because reference data (cgtcalc output)
            // often rounds to nearest integer or uses 5dp, while we use exact decimal.
            let epsilon = Decimal::new(1, 0); // 1.0

            assert!(
                (actual_tax_year.total_gain - first_tax_year.total_gain).abs() <= epsilon,
                "Total Gain mismatch for {}. Actual: {}, Expected: {}",
                input_path.display(),
                actual_tax_year.total_gain,
                first_tax_year.total_gain
            );

            assert!(
                (actual_tax_year.total_loss - first_tax_year.total_loss).abs() <= epsilon,
                "Total Loss mismatch for {}. Actual: {}, Expected: {}",
                input_path.display(),
                actual_tax_year.total_loss,
                first_tax_year.total_loss
            );

            // Verify number of disposals match
            assert_eq!(
                actual_tax_year.disposals.len(),
                first_tax_year.disposals.len(),
                "Disposal count mismatch for {}",
                input_path.display()
            );
        }
    }
}
