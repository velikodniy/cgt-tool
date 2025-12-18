# cgt-format Specification

## Purpose

Provide formatting utilities for currency amounts, decimals, dates, and UK tax year display. All formatting uses UK conventions (£ symbol, DD/MM/YYYY dates, thousands separators).

## Architecture

The crate provides free functions for formatting - no stateful formatters. Functions are named to clearly describe their purpose:

- `format_gbp()` - Format Decimal as GBP with thousands separators and 2dp
- `format_currency_amount()` - Format CurrencyAmount (GBP or foreign) rounded to minor units
- `format_price()` - Format CurrencyAmount as unit price with full precision
- `format_decimal_trimmed()` - Format Decimal removing trailing zeros
- `format_decimal_with_precision()` - Format Decimal with fixed decimal places
- `format_date()` - Format date as DD/MM/YYYY
- `format_tax_year()` - Format UK tax year as "YYYY/YY"

## Requirements

### Requirement: GBP Formatting

The system SHALL format GBP amounts with £ symbol, thousands separators, and 2 decimal places.

#### Scenario: UK currency format

- **WHEN** formatting a Decimal as GBP using `format_gbp()`
- **THEN** display with £ symbol prefix
- **AND** use comma thousands separators (e.g., £1,234.00)
- **AND** always show exactly 2 decimal places

#### Scenario: Negative amounts

- **WHEN** formatting a negative value
- **THEN** display sign before symbol (e.g., -£100.00)

### Requirement: Rounding Policy

The system SHALL round currency values using midpoint-away-from-zero.

#### Scenario: Penny rounding

- **WHEN** rounding £100.995 with `format_gbp()`
- **THEN** result is £101.00 (rounds away from zero)

### Requirement: CurrencyAmount Formatting

The system SHALL provide distinct formatting for totals vs unit prices.

#### Scenario: Format currency amount (totals)

- **WHEN** using `format_currency_amount()` for proceeds, costs, or gains
- **THEN** round to currency minor units
- **AND** use currency symbol (£ for GBP, $ for USD, etc.)
- **AND** for non-GBP, show value with ISO code (e.g., "150.00 USD")

#### Scenario: Format price (unit prices)

- **WHEN** using `format_price()` for per-share prices
- **THEN** preserve full precision
- **AND** strip trailing zeros (e.g., £4.6702, not £4.67020000)
- **AND** use currency symbol when available

### Requirement: Date Formatting

The system SHALL format dates in UK convention (DD/MM/YYYY).

#### Scenario: Date display

- **WHEN** formatting a date with `format_date()`
- **THEN** use day/month/year order with leading zeros

### Requirement: Tax Year Formatting

The system SHALL format UK tax years as "YYYY/YY".

#### Scenario: Tax year display

- **WHEN** formatting tax year 2023 with `format_tax_year(2023)`
- **THEN** display as "2023/24"

### Requirement: Decimal Formatting

The system SHALL provide decimal formatting with configurable precision.

#### Scenario: Fixed precision

- **WHEN** using `format_decimal_with_precision(value, 4)`
- **THEN** always show exactly 4 decimal places

#### Scenario: Trimmed decimals

- **WHEN** using `format_decimal_trimmed(value)`
- **THEN** strip trailing zeros after decimal point
- **AND** remove decimal point if no fractional part

## Implementation Notes

### No Stateful Formatters

Previous design had `CurrencyFormatter` and `FormattingPolicy` structs that were zero-sized and added unnecessary complexity. Current design uses simple free functions.

### Naming Convention

Function names explicitly describe what they format and how:

- `format_gbp` (not `format_currency`) - specific to GBP
- `format_currency_amount` (not `format_amount`) - takes CurrencyAmount type
- `format_price` (not `format_unit`) - clearer for unit prices
- `format_decimal_trimmed` (not `format_decimal`) - describes behavior
- `format_decimal_with_precision` (not `format_decimal_fixed`) - clearer parameter name

### Internal Helpers

Private helper functions use descriptive names:

- `add_thousands_separators()` - not `format_with_commas_str()`
- `format_with_symbol_and_precision()` - not `format_currency_impl()`
