# ci-cd Specification

## Purpose

Automate code quality checks, multi-platform binary releases, HMRC exchange rate updates, and Homebrew tap distribution for the cgt-tool project.

## Requirements

### Requirement: Continuous Integration

The system SHALL run automated checks on every push and pull request to the repository.

#### Scenario: Code quality checks pass

- **WHEN** code is pushed or a PR is opened
- **THEN** the CI pipeline runs all pre-commit checks (trailing whitespace, EOF fixer, YAML validation, large file check, markdown formatting)
- **AND** runs `cargo fmt --check` to verify formatting
- **AND** runs `cargo clippy` with workspace deny rules
- **AND** runs `cargo test` for all crates

#### Scenario: CI blocks merge on failure

- **WHEN** any CI check fails
- **THEN** the workflow reports failure status
- **AND** the PR cannot be merged until fixed

### Requirement: Binary Release Build

The system SHALL build and release binaries for major platforms when a version tag is pushed.

#### Scenario: Tag triggers release

- **WHEN** a tag matching `v*` pattern is pushed
- **THEN** the release workflow builds binaries for Linux x86_64, macOS x86_64, macOS aarch64, Windows x86_64, and Raspberry Pi aarch64

#### Scenario: Artifacts published to GitHub Release

- **WHEN** all platform builds complete successfully
- **THEN** binaries are uploaded to a GitHub Release
- **AND** SHA256 checksums are generated for each binary

### Requirement: Automated FX Rate Updates

The system SHALL automatically download and integrate new HMRC exchange rates monthly.

#### Scenario: Scheduled rate check

- **WHEN** the last Friday of each month arrives (after HMRC penultimate Thursday publication)
- **THEN** the workflow downloads any missing FX rate XML files to `crates/cgt-money/resources/rates/`

#### Scenario: New rates trigger release

- **WHEN** new FX rate files are downloaded
- **THEN** the workflow bumps the minor version in `crates/cgt-money/Cargo.toml`
- **AND** bumps the minor version in `crates/cgt-cli/Cargo.toml`
- **AND** commits changes and creates a new version tag
- **AND** pushes the tag to trigger the release workflow

#### Scenario: Manual trigger available

- **WHEN** a maintainer manually triggers the FX update workflow
- **THEN** the workflow runs the same update process regardless of schedule

### Requirement: Homebrew Tap Release Notification

The system SHALL notify the Homebrew tap repository when a new release is published, triggering an automated formula update.

#### Scenario: Release triggers tap notification

- **WHEN** a GitHub release is published in `velikodniy/cgt-tool`
- **THEN** a `repository_dispatch` event with type `cgt-tool-release` is sent to `velikodniy/homebrew-tap`
- **AND** the event payload includes the release tag name

#### Scenario: Tap workflow receives notification

- **WHEN** the tap repository receives a `cgt-tool-release` dispatch event
- **THEN** the existing `update-cgt-tool.yml` workflow runs with the provided tag
- **AND** the Homebrew formula is updated to the new release version

#### Scenario: Missing token prevents notification

- **WHEN** the `TAP_REPO_TOKEN` secret is not configured
- **THEN** the notification step fails with a clear error message
- **AND** the release itself is not affected (notification is a separate job)

### Requirement: Homebrew Tap Distribution

The system SHALL provide a Homebrew tap formula to install `cgt-tool` from `velikodniy/tap`.

#### Scenario: Install via tap

- **WHEN** a user runs `brew tap velikodniy/tap` followed by `brew install cgt-tool`
- **THEN** installation succeeds on supported macOS architectures (Intel and Apple Silicon) and Linux
- **AND** running `cgt-tool --version` returns the tagged release version.

#### Scenario: Release-sourced binaries

- **WHEN** installing via the tap formula on macOS or Linux
- **THEN** the URL for each architecture matches the corresponding GitHub release asset
- **AND** the formula SHA256 matches the published checksum for that asset.

#### Scenario: Documented install path

- **WHEN** users view installation docs
- **THEN** the cgt-tool README shows `brew tap velikodniy/tap` and `brew install cgt-tool`
- **AND** the tap README lists `cgt-tool` with a tap-install snippet.

### Requirement: Homebrew Formula Automation

The system SHALL keep the tap formula updated automatically when new `cgt-tool` releases are published.

#### Scenario: Release-triggered update

- **WHEN** a newer `velikodniy/cgt-tool` release tag exists than the version in `Formula/cgt-tool.rb`
- **THEN** the workflow updates the formula version, URLs, and SHA256 values to that release
- **AND** commits or opens a pull request in `velikodniy/homebrew-tap` to apply the change.

#### Scenario: Manual rerun

- **WHEN** a maintainer triggers the workflow manually with a target tag
- **THEN** the workflow reuses the same update logic to rewrite the formula for the requested release.

#### Scenario: Checksum integrity

- **WHEN** the workflow prepares a formula update
- **THEN** it sources checksums from the release artifacts (e.g., `checksums.txt` or downloaded assets) for macOS and Linux architectures
- **AND** the run fails if any expected checksum is missing or mismatched.

#### Scenario: Automated smoke install

- **WHEN** the workflow finishes rewriting the formula
- **THEN** it installs `velikodniy/tap/cgt-tool` on macOS
- **AND** `cgt-tool --version` returns the updated release tag before changes are pushed.

### Requirement: Cross-Validation CI Workflow

The system SHALL provide a separate CI workflow for cross-validating cgt-tool calculations against external UK CGT calculators.

#### Scenario: Manual trigger

- **WHEN** a maintainer manually triggers the cross-validate workflow
- **THEN** the workflow runs the same validation as the scheduled run
- **AND** reports results in the workflow summary

#### Scenario: Single-job validation on macOS

- **WHEN** the cross-validate job runs on macos-latest
- **THEN** it builds cgt-tool from source
- **AND** installs Python and uv
- **AND** clones mattjgalloway/cgtcalc and builds with `swift build -c release`
- **AND** runs `python3 scripts/cross-validate.py tests/inputs/*.cgt`
- **AND** compares against both cgt-calc and the built cgtcalc binary

#### Scenario: Discrepancy reporting

- **WHEN** cross-validation finds discrepancies greater than £1 per tax year
- **THEN** the workflow fails with exit code 1
- **AND** reports the specific files and tax years with discrepancies
- **AND** shows the difference amounts

#### Scenario: Skipped operations

- **WHEN** a `.cgt` file contains operations not supported by an external calculator (SPLIT, UNSPLIT, CAPRETURN)
- **THEN** that calculator comparison is skipped for that file
- **AND** the skip is reported in the summary
- **AND** the workflow does not fail due to skipped comparisons

#### Scenario: External tool failure handling

- **WHEN** one external calculator fails to install or run
- **THEN** the workflow reports the failure
- **AND** cgt-tool's own tests are unaffected (separate workflow)
