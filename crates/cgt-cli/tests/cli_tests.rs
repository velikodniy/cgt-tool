use assert_cmd::cargo::cargo_bin_cmd; // This is the macro, use it as cargo_bin_cmd!

#[test]
fn test_cli_parse_fails_without_args() {
    let mut cmd = cargo_bin_cmd!("cgt-cli");
    cmd.assert().failure();
}

#[test]
fn test_cli_report_fails_without_file() {
    let mut cmd = cargo_bin_cmd!("cgt-cli");
    cmd.arg("report").assert().failure();
}

#[test]
fn test_cli_parse_success() {
    let mut cmd = cargo_bin_cmd!("cgt-cli");
    cmd.arg("parse").arg("../../tests/data/Simple.cgt")
        .assert()
        .success();
}
