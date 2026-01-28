## ADDED Requirements

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
