use cgt_core::calculator::calculate;
use cgt_core::models::*;
use cgt_core::parser::parse_file;
use rust_decimal::Decimal;
use std::fs;
use std::path::PathBuf;
use std::str::FromStr;

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

            // Skip unsupported or empty/commented files if necessary
            // e.g. Blank.cgt is empty, should produce empty report

            println!("Testing {}", input_path.display());

            let input_content = fs::read_to_string(&input_path).expect("Failed to read input");
            let transactions = parse_file(&input_content).expect("Failed to parse input");

            let output_content = fs::read_to_string(&output_path).expect("Failed to read output");
            let expected_report: TaxReport =
                serde_json::from_str(&output_content).expect("Failed to parse expected output");

                        let year_start = i32::from_str(expected_report.tax_year.split('/').next().unwrap()).unwrap();

            let actual_report = calculate(transactions, year_start).expect("Failed to calculate");

            // Filter expected matches to the requested tax year
            let start_date = chrono::NaiveDate::from_ymd_opt(year_start, 4, 6).unwrap();
            let end_date = chrono::NaiveDate::from_ymd_opt(year_start + 1, 4, 5).unwrap();

            let expected_matches_in_year: Vec<Match> = expected_report
                .matches
                .iter()
                .filter(|m| m.date >= start_date && m.date <= end_date)
                .cloned()
                .collect();

            let expected_gain: Decimal = expected_matches_in_year
                .iter()
                .map(|m| {
                    if m.gain_or_loss > rust_decimal::Decimal::ZERO {
                        m.gain_or_loss
                    } else {
                        rust_decimal::Decimal::ZERO
                    }
                })
                .sum();

            let expected_loss: Decimal = expected_matches_in_year
                .iter()
                .map(|m| {
                    if m.gain_or_loss < rust_decimal::Decimal::ZERO {
                        m.gain_or_loss.abs()
                    } else {
                        rust_decimal::Decimal::ZERO
                    }
                })
                .sum();

            // Allow larger precision differences because reference data (cgtcalc output)
            // often rounds to nearest integer or uses 5dp, while we use exact decimal.
            let epsilon = Decimal::new(1, 0); // 1.0

            assert!(
                (actual_report.total_gain - expected_gain).abs() <= epsilon,
                "Total Gain mismatch for {}. Actual: {}, Expected: {}",
                input_path.display(),
                actual_report.total_gain,
                expected_gain
            );

            assert!(
                (actual_report.total_loss - expected_loss).abs() <= epsilon,
                "Total Loss mismatch for {}. Actual: {}, Expected: {}",
                input_path.display(),
                actual_report.total_loss,
                expected_loss
            );

            // assert_eq!(actual_report.matches.len(), expected_matches_in_year.len(), "Match count mismatch for {}", input_path.display());
        }
    }
}
