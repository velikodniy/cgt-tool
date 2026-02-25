## ADDED Requirements

### Requirement: Use Core Computed Methods

The PDF formatter SHALL obtain disposal and tax-year derived totals from `cgt-core` model methods rather than re-implementing formulas locally.

#### Scenario: PDF summary and disposal rendering

- **WHEN** PDF output renders disposal totals and annual summary values
- **THEN** the formatter uses `Disposal` and `TaxYearSummary` computed methods for those values
- **AND** output values remain unchanged from prior behavior
