# FX Conversion Specification

## Purpose

Convert foreign currency amounts to GBP using HMRC monthly exchange rates.

## Requirements

### Requirement: HMRC Rates

The system SHALL use HMRC monthly average exchange rates for the transaction month.

#### Scenario: Rate lookup

- **WHEN** converting foreign currency
- **THEN** use HMRC rate for that currency and transaction month

### Requirement: Bundled Rates

The system SHALL include embedded rates from January 2015 to August 2025.

#### Scenario: Default rates

- **WHEN** no custom rate folder provided
- **THEN** use bundled HMRC rates

### Requirement: Custom Rate Folder

The system SHALL load rates from XML files in `--fx-folder`.

#### Scenario: Rate loading

- **WHEN** custom folder specified
- **THEN** load `YYYY-MM.xml` or `monthly_xml_YYYY-MM.xml` files
- **AND** prefer custom rates over bundled when both exist

### Requirement: Missing Rate Errors

The system SHALL fail with clear error when rate is unavailable.

#### Scenario: Missing rate

- **WHEN** currency/month combination has no rate
- **THEN** report missing currency and month with guidance

### Requirement: Dual Display

Reports SHALL show GBP primary with original currency in parentheses.

#### Scenario: Display format

- **WHEN** transaction was in foreign currency
- **THEN** display as `£118.42 (150 USD)` in text, include both in JSON

### Requirement: Precision

The system SHALL use 6 decimal places internally, 2 for display.

#### Scenario: Rounding

- **WHEN** converting and displaying
- **THEN** maintain internal precision, round to 2 decimals for output

### Requirement: GBP Default

The system SHALL treat amounts without currency code as GBP.

#### Scenario: No conversion

- **WHEN** no currency specified or GBP explicit
- **THEN** skip conversion
