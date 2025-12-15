# PDF Formatter Specification (Delta)

## ADDED Requirements

### Requirement: Shared Formatter Dependency

The system SHALL use `cgt-format` for all currency formatting instead of implementing local helpers.

#### Scenario: Currency formatting source

- **WHEN** formatting currency values in PDF output
- **THEN** use `CurrencyFormatter` from `cgt-format`
- **AND** do not implement ad-hoc formatting helpers

### Requirement: Formatting Parity

The system SHALL produce identical currency/date formatting as the plain text formatter.

#### Scenario: Consistent output across formats

- **WHEN** formatting the same value in PDF and plain text
- **THEN** the formatted string is identical
- **AND** both use the same `CurrencyFormatter` implementation
