## ADDED Requirements

### Requirement: Use Core Computed Methods

The plain formatter SHALL obtain disposal and tax-year derived totals from `cgt-core` model methods rather than re-implementing formulas locally.

#### Scenario: Plain summary and disposal rendering

- **WHEN** plain text output renders disposal totals and annual summary values
- **THEN** the formatter uses `Disposal` and `TaxYearSummary` computed methods for those values
- **AND** output values remain unchanged from prior behavior
