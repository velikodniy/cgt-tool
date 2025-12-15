# cgt-format Specification

## Purpose

TBD - created by archiving change restructure-crates. Update Purpose after archive.

## Requirements

### Requirement: Currency Formatting

The system SHALL format currency values with symbol, thousands separators, and appropriate decimal places.

#### Scenario: UK currency format

- **WHEN** formatting a GBP amount
- **THEN** display with £ symbol prefix
- **AND** use comma thousands separators (e.g., £1,234.56)
- **AND** round to 2 decimal places

#### Scenario: Negative amounts

- **WHEN** formatting a negative currency value
- **THEN** display sign before symbol (e.g., -£100.00)

#### Scenario: Foreign currency symbols

- **WHEN** formatting a foreign currency amount
- **THEN** use the currency's native symbol when available
- **AND** fall back to ISO code when symbol is empty

### Requirement: Rounding Policy

The system SHALL round currency values using midpoint-away-from-zero to the currency's minor units.

#### Scenario: Penny rounding

- **WHEN** rounding £100.995 to minor units
- **THEN** result is £101.00 (rounds away from zero)

#### Scenario: Minor unit precision

- **WHEN** formatting for a currency with 2 minor units (GBP)
- **THEN** always display exactly 2 decimal places

### Requirement: Amount vs Unit Formatting

The system SHALL provide separate methods for totals (rounded) and unit prices (full precision).

#### Scenario: Format amount for totals

- **WHEN** formatting proceeds, costs, or gains
- **THEN** round to currency minor units (2dp for GBP)

#### Scenario: Format unit for prices

- **WHEN** formatting unit prices in transaction breakdowns
- **THEN** preserve full precision and strip trailing zeros

### Requirement: Date Formatting

The system SHALL format dates in UK convention (DD/MM/YYYY).

#### Scenario: Date display

- **WHEN** formatting a date
- **THEN** use day/month/year order with leading zeros

### Requirement: Tax Year Formatting

The system SHALL format UK tax years as "YYYY/YY".

#### Scenario: Tax year display

- **WHEN** formatting tax year starting 2023
- **THEN** display as "2023/24"

### Requirement: Decimal Formatting

The system SHALL provide utilities for decimal formatting with configurable precision.

#### Scenario: Fixed precision

- **WHEN** using `format_decimal_fixed(value, 4)`
- **THEN** always show exactly 4 decimal places

#### Scenario: Trimmed decimals

- **WHEN** using `format_decimal(value)`
- **THEN** strip trailing zeros after decimal point

### Requirement: CurrencyFormatter API

The system SHALL provide a `CurrencyFormatter` struct with idiomatic Rust API.

#### Scenario: Formatter construction

- **WHEN** creating a UK formatter
- **THEN** use `CurrencyFormatter::uk()` factory method

#### Scenario: Format CurrencyAmount

- **WHEN** formatting a `CurrencyAmount` with `format_amount()`
- **THEN** return GBP-formatted string with symbol and proper rounding
