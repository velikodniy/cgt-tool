#![allow(clippy::panic)]

use assert_cmd::cargo::cargo_bin_cmd; // This is the macro, use it as cargo_bin_cmd!
use std::fs;
use std::path::Path;

#[test]
fn test_cli_parse_fails_without_args() {
    let mut cmd = cargo_bin_cmd!("cgt-tool");
    cmd.assert().failure();
}

#[test]
fn test_cli_report_fails_without_file() {
    let mut cmd = cargo_bin_cmd!("cgt-tool");
    cmd.arg("report").assert().failure();
}

#[test]
fn test_cli_parse_success() {
    let mut cmd = cargo_bin_cmd!("cgt-tool");
    cmd.arg("parse")
        .arg("../../tests/inputs/Simple.cgt")
        .assert()
        .success();
}

/// Test cases: (name, year)
const PLAIN_FORMAT_TESTS: &[(&str, u16)] = &[
    ("CarryLoss", 2017),
    ("Simple", 2018),
    ("Blank", 2018),
    ("HMRCExample1", 2018),
    ("GainsAndLosses", 2018),
    ("MultipleMatches", 2018),
    ("SameDayMerge", 2018),
    ("SameDayMergeInterleaved", 2018),
    ("SimpleTwoSameDay", 2018),
    ("WithAssetEventsSameDay", 2018),
    ("WithSplitBB", 2018),
    ("WithSplitS104", 2018),
    ("WithUnsplitBB", 2018),
    ("WithUnsplitS104", 2018),
    ("BuySellAllBuyAgainCapitalReturn", 2018),
    ("WithAssetEvents", 2019),
    ("WithAssetEventsBB", 2019),
    ("WithAssetEventsMultipleYears", 2019),
    ("AssetEventsNotFullSale", 2019),
    ("AssetEventsNotFullSale2", 2019),
    ("UnsortedTransactions", 2022),
    ("MultiTickerBasic", 2023),
    ("MultiTickerSameDay", 2023),
    ("MultiTickerBedAndBreakfast", 2023),
    ("MultiTickerSplit", 2023),
    ("2024_2025_SpecialYear", 2024),
];

#[test]
fn test_plain_format_outputs() {
    for (name, year) in PLAIN_FORMAT_TESTS {
        let input_path = format!("../../tests/inputs/{}.cgt", name);
        let expected_path = format!("../../tests/plain/{}.txt", name);

        let expected_output = fs::read_to_string(&expected_path)
            .unwrap_or_else(|e| panic!("Failed to read {}: {}", expected_path, e));

        let mut cmd = cargo_bin_cmd!("cgt-tool");
        let output = cmd
            .arg("report")
            .arg("--year")
            .arg(year.to_string())
            .arg("--format")
            .arg("plain")
            .arg(&input_path)
            .output()
            .unwrap_or_else(|e| panic!("Failed to run CLI for {}: {}", name, e));

        assert!(
            output.status.success(),
            "CLI failed for {}: {}",
            name,
            String::from_utf8_lossy(&output.stderr)
        );

        let actual_output = String::from_utf8_lossy(&output.stdout);
        assert_eq!(
            actual_output, expected_output,
            "Output mismatch for {}",
            name
        );
    }
}

#[test]
fn test_pdf_format_generates_valid_pdfs() {
    let temp_dir = std::env::temp_dir();

    for (name, year) in PLAIN_FORMAT_TESTS {
        let input_path = format!("../../tests/inputs/{}.cgt", name);
        let output_path = temp_dir.join(format!("{}_test.pdf", name));

        // Clean up any existing file
        let _ = fs::remove_file(&output_path);

        let mut cmd = cargo_bin_cmd!("cgt-tool");
        let output = cmd
            .arg("report")
            .arg("--year")
            .arg(year.to_string())
            .arg("--format")
            .arg("pdf")
            .arg("--output")
            .arg(&output_path)
            .arg(&input_path)
            .output()
            .unwrap_or_else(|e| panic!("Failed to run CLI for {}: {}", name, e));

        assert!(
            output.status.success(),
            "CLI failed for {}: {}",
            name,
            String::from_utf8_lossy(&output.stderr)
        );

        // Verify PDF file was created
        assert!(
            Path::new(&output_path).exists(),
            "PDF file was not created for {}",
            name
        );

        // Verify PDF header
        let pdf_content = fs::read(&output_path)
            .unwrap_or_else(|e| panic!("Failed to read PDF for {}: {}", name, e));
        assert!(
            pdf_content.starts_with(b"%PDF"),
            "Invalid PDF header for {}",
            name
        );

        // Clean up
        let _ = fs::remove_file(&output_path);
    }
}

#[test]
fn test_convert_schwab_basic() {
    let mut cmd = cargo_bin_cmd!("cgt-tool");
    cmd.arg("convert")
        .arg("schwab")
        .arg("../cgt-converter/tests/fixtures/schwab/transactions_basic.csv")
        .assert()
        .success()
        .stdout(predicates::str::contains("2023-04-25 BUY XYZZ"))
        .stdout(predicates::str::contains("@ 125.50 USD"))
        .stdout(predicates::str::contains("2023-05-10 SELL XYZZ"))
        .stdout(predicates::str::contains("# Converted from Charles Schwab"));
}

#[test]
fn test_convert_schwab_with_awards() {
    let mut cmd = cargo_bin_cmd!("cgt-tool");
    cmd.arg("convert")
        .arg("schwab")
        .arg("../cgt-converter/tests/fixtures/schwab/transactions_rsu.csv")
        .arg("--awards")
        .arg("../cgt-converter/tests/fixtures/schwab/awards.json")
        .assert()
        .success()
        .stdout(predicates::str::contains("# RSU Vesting"))
        .stdout(predicates::str::contains(
            "2023-04-25 BUY XYZZ 67.2 @ 125.6445",
        ));
}

#[test]
fn test_convert_schwab_rsu_without_awards_fails() {
    let mut cmd = cargo_bin_cmd!("cgt-tool");
    cmd.arg("convert")
        .arg("schwab")
        .arg("../cgt-converter/tests/fixtures/schwab/transactions_rsu.csv")
        .assert()
        .failure();
}
