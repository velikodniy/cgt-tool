## ADDED Requirements

### Requirement: Shared Date and Ticker Sort Usage

The PDF formatter MUST use the canonical date+ticker ordering helper from `cgt-core` for any formatter-level sorting that follows date ascending and ticker ascending semantics.

#### Scenario: Formatter call sites use shared helper

- **WHEN** PDF formatter sorts rows or records by date then ticker
- **THEN** it calls the shared comparator helper instead of defining a local inline comparator
- **AND** visible report ordering is unchanged

#### Scenario: Cross-format ordering parity

- **WHEN** the same report data is rendered in plain text and PDF
- **THEN** both outputs use the same date+ticker ordering behavior
