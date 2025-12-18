## MODIFIED Requirements

### Requirement: Same Day Rule

The system SHALL match disposals with same-day acquisitions first.

Implementation Note: Matching logic is now modularized in `matcher/same_day.rs`.

#### Scenario: Same day match

- **WHEN** shares are bought and sold on the same day
- **THEN** match sale against same-day purchases before other rules
- **AND** aggregate multiple same-day purchases if needed
