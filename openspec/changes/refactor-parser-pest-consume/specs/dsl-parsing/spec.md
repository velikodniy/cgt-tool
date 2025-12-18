# DSL Parsing Specification - Delta

## MODIFIED Requirements

### Requirement: Parser Implementation

The system SHALL use `pest_consume` for parsing the DSL grammar with semantic parsing methods.

#### Scenario: Grammar structure

- **WHEN** the parser is initialized
- **THEN** it uses semantic types that embed keywords (e.g., `price = { "@" ~ money }`, `fees = { ^"FEES" ~ money }`)
- **AND** it uses atomic rules (with `@` prefix) for terminal tokens like `date`, `ticker`, `quantity`
- **AND** it implements semantic parsing methods for each grammar rule

#### Scenario: Error handling

- **WHEN** parsing fails
- **THEN** `pest_consume::Error` is converted to `CgtError::ParseError`
- **AND** error messages include line number, column, and expected input

#### Scenario: Backward compatibility

- **WHEN** any valid DSL file from previous parser is parsed
- **THEN** it produces identical transaction data structures
- **AND** all existing test fixtures pass without modification
