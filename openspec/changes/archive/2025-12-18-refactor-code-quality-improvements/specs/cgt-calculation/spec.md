## MODIFIED Requirements

### Requirement: Section 104 Pool

The system SHALL maintain pooled holdings at average cost for remaining shares.

#### Scenario: Pool operations

- **WHEN** disposing shares not matched by Same Day or B&B
- **THEN** use average cost from Section 104 pool
- **AND** update pool on purchases (add) and sales (reduce proportionally)

#### Scenario: Zero sell amount guard

- **WHEN** a disposal has zero total sell amount (edge case)
- **THEN** return no match result without attempting division
- **AND** proceed to next matching rule

## ADDED Requirements

### Requirement: Validation Contract

The system SHALL document that transaction validation should occur before calculation.

#### Scenario: Calculation without prior validation

- **WHEN** `calculate()` is called without prior `validate()` call
- **THEN** calculation proceeds but may encounter unexpected behavior on invalid inputs
- **AND** documentation warns that `validate()` should be called first

#### Scenario: Explicit validated calculation

- **WHEN** caller wants guaranteed validation
- **THEN** caller can call `validate()` first and check `is_valid()` before proceeding
- **AND** this pattern is documented in function doc comments
