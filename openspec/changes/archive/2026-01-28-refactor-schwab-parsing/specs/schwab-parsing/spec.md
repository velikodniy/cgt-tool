## ADDED Requirements

### Requirement: Polymorphic Transaction Parsing

The system SHALL parse Schwab transactions based on the "Action" field, validating required fields for each known type.

#### Scenario: Parse Buy Transaction

- **WHEN** parsing a transaction with Action "Buy"
- **THEN** it MUST require "Quantity", "Price", "Date", and "Symbol" to be present and valid
- **AND** it SHALL parse "Quantity" and "Price" directly into Decimal types

#### Scenario: Parse Unknown Transaction

- **WHEN** parsing a transaction with an Action not explicitly defined in the enum
- **THEN** it SHALL be captured as an "Unknown" record variant
- **AND** it SHALL NOT cause the file parsing to fail
- **AND** it SHALL NOT require "Quantity" or "Price" to be present

### Requirement: Output Comments for Skipped Items

The system SHALL document skipped or non-impactful transactions in the generated output file as comments, rather than as console warnings.

#### Scenario: Skipped Unknown Entry

- **WHEN** processing an unknown transaction
- **THEN** the generated .cgt file SHALL contain a comment line indicating it was skipped
- **AND** no warning SHALL appear in the CLI stderr
