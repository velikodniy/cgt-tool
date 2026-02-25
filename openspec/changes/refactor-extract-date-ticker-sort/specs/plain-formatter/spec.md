## ADDED Requirements

### Requirement: Shared Date and Ticker Sort Usage

The plain formatter MUST use the canonical date+ticker ordering helper from `cgt-core` for any formatter-level sorting that follows date ascending and ticker ascending semantics.

#### Scenario: Formatter call sites use shared helper

- **WHEN** plain formatter sorts rows or records by date then ticker
- **THEN** it calls the shared comparator helper instead of defining a local inline comparator
- **AND** visible report ordering is unchanged

#### Scenario: Multi-currency report ordering stability

- **WHEN** plain formatter renders transactions across multiple tickers and dates
- **THEN** ordering remains deterministic regardless of currency conversion details
