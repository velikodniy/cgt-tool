## MODIFIED Requirements

### Requirement: Calculate Report Tool

The system SHALL provide `calculate_report` tool to generate CGT reports.

#### Scenario: Generate report for specific tax year

- **WHEN** tool is called with `cgt_content` and `year` (e.g., 2024)
- **THEN** return JSON report for tax year 2024/25 only
- **AND** include gains, losses, disposals, matches, and pool states

#### Scenario: Generate report for all years

- **WHEN** tool is called with `cgt_content` and `year` omitted
- **THEN** return JSON report for all tax years with disposals
- **AND** sort `tax_years` chronologically (earliest first)

#### Scenario: Report with FX conversion

- **WHEN** cgt_content contains foreign currency transactions
- **THEN** convert using bundled HMRC rates
- **AND** include both GBP and original currency in output

#### Scenario: No disposals in year

- **WHEN** year parameter results in no disposals
- **THEN** return report with zero gains/losses (not an error)

#### Scenario: No disposals at all

- **WHEN** cgt_content has no SELL transactions
- **THEN** return report with empty `tax_years` array and current holdings
