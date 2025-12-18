## MODIFIED Requirements

### Requirement: Error Handling

The system SHALL return structured errors compatible with MCP error protocol. Error types SHALL only include variants that are actually used.

#### Scenario: Tool execution error

- **WHEN** a tool fails due to invalid input or calculation error
- **THEN** return JSON-RPC error with code and descriptive message
- **AND** include relevant context (line numbers, invalid values)
- **AND** include hints for fixing common errors
- **AND** include examples of valid input format

#### Scenario: Protocol error

- **WHEN** client sends malformed JSON-RPC or unknown method
- **THEN** return appropriate JSON-RPC error code (-32700, -32601, etc.)

#### Scenario: Missing currency in object format

- **WHEN** price/fees is provided as object `{"amount": "X"}` without currency field
- **THEN** return error explaining that object format requires explicit currency
- **AND** suggest using plain string for GBP or adding currency field

#### Scenario: Disposal not found in explain_matching

- **WHEN** no disposal exists for given date and ticker in explain_matching tool
- **THEN** return MCP invalid_params error with helpful message
- **AND** list available disposals for that ticker if any exist

## ADDED Requirements

### Requirement: Resource Organization

The system SHALL organize string constants and documentation in a dedicated resources module for maintainability.

#### Scenario: Hint messages in resources

- **WHEN** server needs to display hint messages for errors
- **THEN** hint constants are defined in resources module
- **AND** server imports hints from resources module

#### Scenario: DSL syntax reference in resources

- **WHEN** server needs to display DSL syntax help
- **THEN** DSL syntax reference is defined in resources module
- **AND** server imports DSL syntax from resources module

#### Scenario: Example transactions in resources

- **WHEN** server needs to display example transactions
- **THEN** example transaction strings are defined in resources module
- **AND** server imports examples from resources module
