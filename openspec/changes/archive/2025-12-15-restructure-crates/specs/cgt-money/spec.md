# Currency Primitives Specification (cgt-money)

## Purpose

Provide currency primitives and FX conversion for UK CGT calculations. This crate owns `CurrencyAmount`, FX rate loading/caching, and re-exports `iso_currency`.

## ADDED Requirements

### Requirement: CurrencyAmount Type

The system SHALL provide a `CurrencyAmount` type that stores original amount, currency, and GBP equivalent.

#### Scenario: GBP amount

- **WHEN** creating a GBP amount
- **THEN** `amount` and `gbp` fields are equal
- **AND** `is_gbp()` returns true

#### Scenario: Foreign currency amount

- **WHEN** creating a foreign currency amount
- **THEN** `amount` stores the original value
- **AND** `gbp` stores the converted GBP value
- **AND** `is_gbp()` returns false

#### Scenario: Currency metadata

- **WHEN** querying a `CurrencyAmount`
- **THEN** `minor_units()` returns the currency's decimal places (e.g., 2 for GBP)
- **AND** `symbol()` returns the currency symbol (e.g., "£")
- **AND** `code()` returns the ISO code (e.g., "GBP")

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
