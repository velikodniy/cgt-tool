## ADDED Requirements

### Requirement: Canonical Date and Ticker Ordering

The system MUST provide a single canonical ordering helper for records sorted by transaction date and ticker symbol, and calculation code SHALL use that shared helper rather than duplicate comparator logic.

#### Scenario: Calculator uses shared comparator

- **WHEN** calculator logic sorts disposal-like records by date and ticker
- **THEN** it uses the canonical shared comparator from `cgt-core`
- **AND** sorting order remains date ascending, then ticker ascending

#### Scenario: Error handling and determinism

- **WHEN** sorting records with equal date and ticker values
- **THEN** sorting completes without panic or unwrap-based failure
- **AND** downstream tax-year grouping remains deterministic
