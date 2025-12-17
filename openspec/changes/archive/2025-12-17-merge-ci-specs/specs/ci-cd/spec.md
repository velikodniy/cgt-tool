# ci-cd Specification

## ADDED Requirements

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
