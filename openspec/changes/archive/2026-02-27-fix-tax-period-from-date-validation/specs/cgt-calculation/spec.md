## MODIFIED Requirements

### Requirement: Tax Year Grouping

The system SHALL group disposals by UK tax year (6 April to 5 April). `TaxPeriod::from_date()` SHALL return `Result<Self, CgtError>` and enforce the same `[1900, 2100]` range validation as `TaxPeriod::new()`.

#### Scenario: Year boundaries

- **WHEN** disposal is on 5 April 2024
- **THEN** assign to 2023/24; 6 April 2024 starts 2024/25

#### Scenario: Single year calculation

- **WHEN** `calculate()` is called with a specific `tax_year_start`
- **THEN** return `TaxReport` containing only that tax year's disposals
- **AND** filter out disposals from other years

#### Scenario: All years calculation

- **WHEN** `calculate()` is called with `None` for `tax_year_start`
- **THEN** return `TaxReport` containing all tax years with disposals
- **AND** group disposals into separate `TaxYearSummary` entries by tax period
- **AND** sort `tax_years` chronologically (earliest first)

#### Scenario: Date outside valid tax year range

- **WHEN** `TaxPeriod::from_date()` is called with a date that resolves to a year outside `[1900, 2100]`
- **THEN** it SHALL return `Err(CgtError::InvalidTaxYear(_))`
