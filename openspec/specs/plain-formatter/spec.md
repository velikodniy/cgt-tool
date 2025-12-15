# Plain Text Formatter Specification

## Purpose

Generate human-readable plain text CGT reports compatible with cgtcalc format.

## Requirements

### Requirement: Report Sections

The system SHALL output: SUMMARY, TAX YEAR DETAILS, HOLDINGS, TRANSACTIONS.

#### Scenario: Section structure

- **WHEN** generating plain text report
- **THEN** include all sections in order with clear headings

### Requirement: Summary Table

The system SHALL show tax year, gain, proceeds, exemption, taxable gain per year.

#### Scenario: Multi-year summary

- **WHEN** disposals span multiple tax years
- **THEN** show one row per tax year

### Requirement: Disposal Details

The system SHALL list each disposal with matching rule breakdown.

#### Scenario: Match display

- **WHEN** disposal uses Same Day, B&B, or Section 104
- **THEN** show rule name, matched quantity, cost, and gain/loss

### Requirement: Holdings Display

The system SHALL list remaining holdings with ticker, quantity, and average cost.

#### Scenario: Holdings section

- **WHEN** holdings remain
- **THEN** show each ticker; show "NONE" if empty

### Requirement: Currency Formatting

The system SHALL use UK formatting: £ symbol, 2 decimal places, DD/MM/YYYY dates.

#### Scenario: Foreign currency

- **WHEN** transaction was in foreign currency
- **THEN** display as `£118.42 (150 USD)`

### Requirement: Proceeds Breakdown

The plain text disposal calculation SHALL display net proceeds using the explicit formula of quantity × unit price minus sale expenses, and the shown net figure SHALL match the calculated proceeds.

#### Scenario: Sell with sale expenses

- **WHEN** a disposal includes sale expenses
- **THEN** the proceeds line is shown as `<quantity> × £<unit price> - £<sale expenses> = £<net proceeds>`
- **AND** `<net proceeds>` equals `(unit price × quantity) - sale expenses` rounded using the currency’s minor units (per the currency policy).

#### Scenario: Sell without sale expenses

- **WHEN** sale expenses are zero
- **THEN** the proceeds line is shown as `<quantity> × £<unit price> = £<net proceeds>`
- **AND** `<net proceeds>` equals `unit price × quantity` rounded using the currency’s minor units (per the currency policy).
- **AND** the proceeds line omits the `- £0` term when sale expenses are zero.

### Requirement: Shared Formatter Dependency

The system SHALL use `cgt-format` for all currency formatting instead of implementing local helpers.

#### Scenario: Currency formatting source

- **WHEN** formatting currency values in plain text output
- **THEN** use `CurrencyFormatter` from `cgt-format`
- **AND** do not implement ad-hoc formatting helpers

### Requirement: Unit Price Precision

The system SHALL use `CurrencyFormatter::format_unit()` for unit prices in transaction breakdowns.

#### Scenario: Unit price in proceeds breakdown

- **WHEN** displaying unit prices in proceeds breakdown
- **THEN** use `format_unit()` to preserve full precision
- **AND** do not create local helper functions for this purpose
