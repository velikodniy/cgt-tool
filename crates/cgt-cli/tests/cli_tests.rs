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

/// Test cases - run without year filter to get all tax years
const PLAIN_FORMAT_TESTS: &[&str] = &[
    "CarryLoss",
    "Simple",
    "Blank",
    "HMRCExample1",
    "GainsAndLosses",
    "MultipleMatches",
    "SameDayMerge",
    "SameDayMergeInterleaved",
    "SimpleTwoSameDay",
    "WithAssetEventsSameDay",
    "WithSplitBB",
    "WithSplitS104",
    "WithUnsplitBB",
    "WithUnsplitS104",
    "BuySellAllBuyAgainCapitalReturn",
    "WithAssetEvents",
    "WithAssetEventsBB",
    "WithAssetEventsMultipleYears",
    "AssetEventsNotFullSale",
    "AssetEventsNotFullSale2",
    "UnsortedTransactions",
    "MultiTickerBasic",
    "MultiTickerSameDay",
    "MultiTickerBedAndBreakfast",
    "MultiTickerSplit",
    "2024_2025_SpecialYear",
    "RateSplit2024",
    "AccumulationDividend",
    "BnBReportQuantity",
    "CapReturnEqualisation",
    "DividendAfterFullDisposal",
    "ExpensesRounding",
    "WhitespaceDividend",
];

#[test]
fn test_plain_format_outputs() {
    for name in PLAIN_FORMAT_TESTS {
        let input_path = format!("../../tests/inputs/{}.cgt", name);
        let expected_path = format!("../../tests/plain/{}.txt", name);

        let expected_output = fs::read_to_string(&expected_path)
            .unwrap_or_else(|e| panic!("Failed to read {}: {}", expected_path, e));

        let mut cmd = cargo_bin_cmd!("cgt-tool");
        let output = cmd
            .arg("report")
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

    for name in PLAIN_FORMAT_TESTS {
        let input_path = format!("../../tests/inputs/{}.cgt", name);
        let output_path = temp_dir.join(format!("{}_test.pdf", name));

        // Clean up any existing file
        let _ = fs::remove_file(&output_path);

        let mut cmd = cargo_bin_cmd!("cgt-tool");
        let output = cmd
            .arg("report")
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

#[test]
fn test_parse_multiple_files() {
    // Create temp files with simple transactions
    let temp_dir = std::env::temp_dir();
    let file1 = temp_dir.join("multi_parse_test1.cgt");
    let file2 = temp_dir.join("multi_parse_test2.cgt");

    fs::write(&file1, "2018-01-01 BUY AAPL 10 @ 100.00\n").expect("Failed to write file1");
    fs::write(&file2, "2018-01-02 BUY AAPL 5 @ 110.00\n").expect("Failed to write file2");

    let mut cmd = cargo_bin_cmd!("cgt-tool");
    let output = cmd
        .arg("parse")
        .arg(&file1)
        .arg(&file2)
        .output()
        .expect("Failed to run CLI");

    assert!(output.status.success(), "CLI should succeed");

    let stdout = String::from_utf8_lossy(&output.stdout);
    // Should contain both transactions
    assert!(stdout.contains("2018-01-01"), "Should contain first date");
    assert!(stdout.contains("2018-01-02"), "Should contain second date");

    // Clean up
    let _ = fs::remove_file(&file1);
    let _ = fs::remove_file(&file2);
}

#[test]
fn test_report_multiple_files() {
    // Create temp files with transactions for same tax year
    let temp_dir = std::env::temp_dir();
    let file1 = temp_dir.join("multi_report_test1.cgt");
    let file2 = temp_dir.join("multi_report_test2.cgt");

    // File 1: Buy transaction
    fs::write(&file1, "2018-08-01 BUY AAPL 10 @ 100.00\n").expect("Failed to write file1");
    // File 2: Sell transaction
    fs::write(&file2, "2018-08-02 SELL AAPL 10 @ 110.00\n").expect("Failed to write file2");

    let mut cmd = cargo_bin_cmd!("cgt-tool");
    let output = cmd
        .arg("report")
        .arg("--year")
        .arg("2018")
        .arg("--format")
        .arg("plain")
        .arg(&file1)
        .arg(&file2)
        .output()
        .expect("Failed to run CLI");

    assert!(
        output.status.success(),
        "CLI should succeed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let stdout = String::from_utf8_lossy(&output.stdout);
    // Should show a gain from the combined transactions
    assert!(stdout.contains("AAPL"), "Should contain ticker");
    assert!(stdout.contains("2018/19"), "Should show tax year");

    // Clean up
    let _ = fs::remove_file(&file1);
    let _ = fs::remove_file(&file2);
}

#[test]
fn test_pdf_multiple_files_default_filename() {
    let temp_dir = std::env::temp_dir();
    let file1 = temp_dir.join("pdf_multi_test1.cgt");
    let file2 = temp_dir.join("pdf_multi_test2.cgt");
    let expected_output = temp_dir.join("report.pdf");

    // Clean up any existing file first
    let _ = fs::remove_file(&expected_output);

    fs::write(&file1, "2018-08-01 BUY AAPL 10 @ 100.00\n").expect("Failed to write file1");
    fs::write(&file2, "2018-08-02 SELL AAPL 10 @ 110.00\n").expect("Failed to write file2");

    let mut cmd = cargo_bin_cmd!("cgt-tool");
    let output = cmd
        .arg("report")
        .arg("--year")
        .arg("2018")
        .arg("--format")
        .arg("pdf")
        .arg(&file1)
        .arg(&file2)
        .current_dir(&temp_dir)
        .output()
        .expect("Failed to run CLI");

    assert!(
        output.status.success(),
        "CLI should succeed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("report.pdf"),
        "Should output to report.pdf for multiple files"
    );

    // Clean up
    let _ = fs::remove_file(&file1);
    let _ = fs::remove_file(&file2);
    let _ = fs::remove_file(&expected_output);
}

#[test]
fn test_pdf_overwrite_protection() {
    let temp_dir = std::env::temp_dir();
    let input_file = temp_dir.join("pdf_overwrite_test.cgt");
    let existing_pdf = temp_dir.join("pdf_overwrite_test.pdf");

    fs::write(
        &input_file,
        "2018-08-01 BUY AAPL 10 @ 100.00\n2018-08-02 SELL AAPL 10 @ 110.00\n",
    )
    .expect("Failed to write input file");

    // Create an existing PDF file that should not be overwritten
    fs::write(&existing_pdf, "existing content").expect("Failed to create existing PDF");

    let mut cmd = cargo_bin_cmd!("cgt-tool");
    let output = cmd
        .arg("report")
        .arg("--year")
        .arg("2018")
        .arg("--format")
        .arg("pdf")
        .arg(&input_file)
        .current_dir(&temp_dir)
        .output()
        .expect("Failed to run CLI");

    // Should fail because file exists
    assert!(
        !output.status.success(),
        "CLI should fail when output file exists"
    );

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("already exists"),
        "Should mention file exists: {}",
        stderr
    );
    assert!(
        stderr.contains("--output"),
        "Should suggest using --output: {}",
        stderr
    );

    // Verify existing file was not modified
    let content = fs::read_to_string(&existing_pdf).expect("Failed to read existing PDF");
    assert_eq!(content, "existing content", "File should not be modified");

    // Clean up
    let _ = fs::remove_file(&input_file);
    let _ = fs::remove_file(&existing_pdf);
}
