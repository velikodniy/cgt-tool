use assert_cmd::Command;
use std::fs;

#[test]
fn test_cli_parse_fails_without_args() {
    let mut cmd = Command::cargo_bin("cgt-cli").unwrap();
    cmd.assert().failure();
}

#[test]
fn test_cli_report_fails_without_file() {
    let mut cmd = Command::cargo_bin("cgt-cli").unwrap();
    cmd.arg("report").assert().failure();
}

#[test]
fn test_cli_parse_success() {
    let mut cmd = Command::cargo_bin("cgt-cli").unwrap();
    cmd.arg("parse")
        .arg("../../tests/data/Simple.cgt")
        .assert()
        .success();
}
