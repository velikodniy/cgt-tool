# DSL Parsing Specification

## Purpose

Parse transaction files in custom DSL format into structured data for CGT calculation.

## Requirements

### Requirement: Transaction Types

The system SHALL parse: BUY, SELL, DIVIDEND, CAPRETURN, SPLIT, UNSPLIT.

#### Scenario: Parse BUY/SELL

- **WHEN** a line contains `YYYY-MM-DD BUY|SELL TICKER QUANTITY @ PRICE [FEES AMOUNT]`
- **THEN** extract all fields into a transaction record
- **AND** default FEES to 0 when omitted

#### Scenario: Parse DIVIDEND

- **WHEN** a line contains `YYYY-MM-DD DIVIDEND TICKER QUANTITY TOTAL VALUE [TAX AMOUNT]`
- **THEN** extract dividend details including optional tax withheld
- **AND** default TAX to 0 when omitted

#### Scenario: Parse CAPRETURN

- **WHEN** a line contains `YYYY-MM-DD CAPRETURN TICKER QUANTITY TOTAL VALUE [FEES AMOUNT]`
- **THEN** extract capital return details
- **AND** default FEES to 0 when omitted

#### Scenario: Parse SPLIT/UNSPLIT

- **WHEN** a line contains `YYYY-MM-DD SPLIT|UNSPLIT TICKER RATIO VALUE`
- **THEN** extract corporate action with ratio

### Requirement: Currency Codes

The system SHALL accept optional ISO 4217 currency codes after monetary amounts.

#### Scenario: Foreign currency

- **WHEN** a transaction includes `@ 150.00 USD`
- **THEN** parse amount with currency; default to GBP if omitted

#### Scenario: Invalid currency

- **WHEN** an unrecognized code is used
- **THEN** report error with invalid code and line number

### Requirement: Comments

The system SHALL ignore lines starting with `#` and inline `# comments`.

#### Scenario: Comment handling

- **WHEN** a line starts with `#` or contains `# comment` after data
- **THEN** skip comment content, parse remaining data if present

### Requirement: Error Messages

The system SHALL report errors with line numbers and expected format.

#### Scenario: Parse failure

- **WHEN** parsing fails (bad date, missing field, invalid number)
- **THEN** report line number, problematic value, and expected format

### Requirement: Ticker Normalization

The system SHALL normalize ticker symbols to uppercase.

#### Scenario: Case normalization

- **WHEN** ticker is `aapl` or `AaPl`
- **THEN** normalize to `AAPL`
