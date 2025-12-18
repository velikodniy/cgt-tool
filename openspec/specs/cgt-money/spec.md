# cgt-money Specification

## Purpose

Specify money primitives, precision, and exchange-rate handling, including bundled and custom HMRC rates for converting foreign amounts to GBP.

## Requirements

### Requirement: CurrencyAmount Type

The system SHALL provide a `CurrencyAmount` type that stores original amount and currency, with on-demand GBP conversion.

#### Scenario: GBP amount

- **WHEN** creating a GBP amount
- **THEN** `amount` stores the value
- **AND** `is_gbp()` returns true
- **AND** `to_gbp()` returns the amount unchanged

#### Scenario: Foreign currency amount

- **WHEN** creating a foreign currency amount
- **THEN** `amount` stores the original value
- **AND** `is_gbp()` returns false
- **AND** `to_gbp(date, fx_cache)` converts using HMRC rate for that month

#### Scenario: Currency metadata

- **WHEN** querying a `CurrencyAmount`
- **THEN** `minor_units()` returns the currency's decimal places (e.g., 2 for GBP)
- **AND** `symbol()` returns the currency symbol (e.g., "£")
- **AND** `code()` returns the ISO code (e.g., "GBP")

#### Scenario: JSON deserialization

- **WHEN** deserializing from JSON
- **THEN** only `amount` field is required
- **AND** `currency` field defaults to "GBP" if not provided

#### Scenario: JSON serialization

- **WHEN** serializing a `CurrencyAmount`
- **THEN** the output includes `amount` and `currency` only (no `gbp` field)

#### Scenario: Conversion error

- **WHEN** calling `to_gbp()` for a foreign currency
- **AND** no FX rate exists for that currency/month
- **THEN** return an error with the missing currency and month

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

### Requirement: Precision

The system SHALL use 6 decimal places internally for FX calculations.

#### Scenario: Internal precision

- **WHEN** converting currency
- **THEN** maintain 6 decimal places internally

### Requirement: Re-export iso_currency

The system SHALL re-export the `Currency` type from `iso_currency`.

#### Scenario: Single import

- **WHEN** using currency types
- **THEN** import from `cgt_money` without separate `iso_currency` dependency
