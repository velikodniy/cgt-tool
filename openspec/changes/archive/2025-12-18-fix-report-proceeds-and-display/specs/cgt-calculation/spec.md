## MODIFIED Requirements

### Requirement: Gain/Loss Calculation

The system SHALL calculate gain or loss as: net proceeds - allowable cost, where net proceeds = gross proceeds - sale expenses.

#### Scenario: Calculation

- **WHEN** disposal is matched
- **THEN** compute gain (positive) or loss (negative)
- **AND** the Disposal struct contains both gross_proceeds and proceeds (net) fields

#### Scenario: Disposal data structure

- **WHEN** a disposal is created
- **THEN** the Disposal struct contains:
  - `gross_proceeds`: The total sale amount before fees (quantity × unit price)
  - `proceeds`: The net sale amount after fees (gross_proceeds - sale fees)
- **AND** both values are available to formatters for display
