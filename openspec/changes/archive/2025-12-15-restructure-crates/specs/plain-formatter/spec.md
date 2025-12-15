# Plain Text Formatter Specification (Delta)

## ADDED Requirements

### Requirement: Shared Formatter Dependency

The system SHALL use `cgt-format` for all currency formatting instead of implementing local helpers.

#### Scenario: Currency formatting source

- **WHEN** formatting currency values in plain text output
- **THEN** use `CurrencyFormatter` from `cgt-format`
- **AND** do not implement ad-hoc formatting helpers

### Requirement: Unit Price Precision

The system SHALL use `CurrencyFormatter::format_unit()` for unit prices in transaction breakdowns.

#### Scenario: Unit price in proceeds breakdown

- **WHEN** displaying unit prices in proceeds breakdown
- **THEN** use `format_unit()` to preserve full precision
- **AND** do not create local helper functions for this purpose
