## MODIFIED Requirements

### Requirement: Report Command

The system SHALL provide `report` to generate CGT reports.

#### Scenario: Generate report for specific year

- **WHEN** `cgt-tool report file.cgt --year 2024` is run
- **THEN** generate report for tax year 2024/25 only

#### Scenario: Generate report for all years

- **WHEN** `cgt-tool report file.cgt` is run without `--year`
- **THEN** generate report including all tax years that contain disposals
- **AND** sort tax years chronologically in output

#### Scenario: Generate report from multiple files

- **WHEN** `cgt-tool report file1.cgt file2.cgt --year 2024` is run
- **THEN** concatenate file contents in argument order
- **AND** parse the combined content
- **AND** generate report for tax year 2024/25

#### Scenario: Generate all-years report from multiple files

- **WHEN** `cgt-tool report file1.cgt file2.cgt` is run without `--year`
- **THEN** concatenate file contents in argument order
- **AND** parse the combined content
- **AND** generate report for all tax years with disposals
