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

The system SHALL show tax year, disposal count, net gain, total gains (before losses), total losses, gross proceeds (for SA108 Box 21), exemption, and taxable gain per year.

#### Scenario: Multi-year summary

- **WHEN** disposals span multiple tax years
- **THEN** show one row per tax year with gross proceeds

#### Scenario: Gross proceeds for SA108

- **WHEN** generating summary table
- **THEN** the Proceeds column displays gross proceeds (before sell fees)
- **AND** this value corresponds to SA108 Box 21 "Disposal proceeds"

#### Scenario: Summary totals and counts

- **WHEN** generating the summary table for any tax year
- **THEN** the disposal count equals the number of grouped disposals in that year after same-day aggregation (CG51560)
- **AND** the gains column equals the tax year's `total_gain`
- **AND** the losses column equals the tax year's `total_loss`

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

The plain text disposal calculation SHALL display both gross and net proceeds with clear labels, and the net figure SHALL match the gain/loss calculation.

#### Scenario: Sell with sale expenses

- **WHEN** a disposal includes sale expenses
- **THEN** the proceeds section shows gross proceeds as `<quantity> × £<unit price> = £<gross proceeds>`
- **AND** shows net proceeds as `£<gross proceeds> - £<sale expenses> fees = £<net proceeds>`
- **AND** `<net proceeds>` equals `(unit price × quantity) - sale expenses`

#### Scenario: Sell without sale expenses

- **WHEN** sale expenses are zero
- **THEN** the proceeds section shows `<quantity> × £<unit price> = £<gross proceeds>`
- **AND** gross and net proceeds are equal
- **AND** the fees line is omitted

#### Scenario: Same-day merge with multiple sells

- **WHEN** multiple sells occur on the same day for the same ticker
- **THEN** display the weighted average unit price calculated as total_gross_proceeds / total_quantity
- **AND** the displayed price reflects the actual average, not an arbitrary individual sell price

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

### Requirement: SA108 Guidance Note

The system SHALL include a brief note in the report explaining which values correspond to SA108 boxes.

#### Scenario: Summary section note

- **WHEN** generating a report with disposals
- **THEN** include a note below the summary table indicating that Proceeds = SA108 Box 21, and that allowable costs include all fees
- **AND** clarify that gains and losses are reported as net per-disposal results after applying matching rules (CG51560)
