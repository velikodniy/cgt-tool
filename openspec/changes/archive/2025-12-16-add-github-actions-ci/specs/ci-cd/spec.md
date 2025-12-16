## ADDED Requirements

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
