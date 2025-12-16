## MODIFIED Requirements

### Requirement: Parse Command

The system SHALL provide `parse` to validate and output transactions as JSON.

#### Scenario: Parse file

- **WHEN** `cgt-tool parse file.cgt` is run
- **THEN** output parsed transactions as JSON to stdout
- **AND** output errors with line numbers to stderr if invalid

### Requirement: Report Command

The system SHALL provide `report` to generate CGT reports.

#### Scenario: Generate report

- **WHEN** `cgt-tool report file.cgt --year 2024` is run
- **THEN** generate report for tax year 2024/25
