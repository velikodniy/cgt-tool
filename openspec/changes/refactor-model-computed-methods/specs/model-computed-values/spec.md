## ADDED Requirements

### Requirement: Disposal Computed Methods

The system SHALL expose computed methods on `Disposal` for formatter-reused derived values.

#### Scenario: Disposal aggregate values

- **WHEN** a `Disposal` contains one or more match rows
- **THEN** `Disposal::net_gain_or_loss()` returns the sum of all match `gain_or_loss` values
- **AND** `Disposal::total_allowable_cost()` returns the sum of all match `allowable_cost` values

### Requirement: Tax Year Computed Methods

The system SHALL expose computed methods on `TaxYearSummary` for formatter-reused derived values.

#### Scenario: Tax year aggregate values

- **WHEN** a `TaxYearSummary` contains disposal entries and has annual exemption data
- **THEN** `TaxYearSummary::gross_proceeds()` returns the sum of each disposal's `gross_proceeds`
- **AND** taxable gain uses the formula `max(net_gain - annual_exempt_amount, 0)` via a model method
