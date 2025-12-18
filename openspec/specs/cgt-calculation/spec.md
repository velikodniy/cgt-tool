# CGT Calculation Specification

## Purpose

Calculate UK Capital Gains Tax using HMRC share matching rules (CG51500-CG51600).

## Requirements

### Requirement: Same Day Rule

The system SHALL match disposals with same-day acquisitions first.

Implementation Note: Matching logic is now modularized in `matcher/same_day.rs`.

#### Scenario: Same day match

- **WHEN** shares are bought and sold on the same day
- **THEN** match sale against same-day purchases before other rules
- **AND** aggregate multiple same-day purchases if needed

### Requirement: Bed and Breakfast Rule

The system SHALL match disposals with acquisitions within 30 days after sale.

#### Scenario: B&B match

- **WHEN** shares are repurchased within 30 days after sale
- **THEN** match chronologically (earliest purchase first)
- **AND** skip if repurchase is beyond 30 days

### Requirement: Section 104 Pool

The system SHALL maintain pooled holdings at average cost for remaining shares.

#### Scenario: Pool operations

- **WHEN** disposing shares not matched by Same Day or B&B
- **THEN** use average cost from Section 104 pool
- **AND** update pool on purchases (add) and sales (reduce proportionally)

#### Scenario: Zero sell amount guard

- **WHEN** a disposal has zero total sell amount (edge case)
- **THEN** return no match result without attempting division
- **AND** proceed to next matching rule

### Requirement: Multi-Ticker Isolation

The system SHALL maintain separate Section 104 pools per ticker.

#### Scenario: Ticker independence

- **WHEN** transactions span multiple tickers
- **THEN** each ticker has independent pool and matching

### Requirement: Tax Year Grouping

The system SHALL group disposals by UK tax year (6 April to 5 April).

#### Scenario: Year boundaries

- **WHEN** disposal is on 5 April 2024
- **THEN** assign to 2023/24; 6 April 2024 starts 2024/25

#### Scenario: Single year calculation

- **WHEN** `calculate()` is called with a specific `tax_year_start`
- **THEN** return `TaxReport` containing only that tax year's disposals
- **AND** filter out disposals from other years

#### Scenario: All years calculation

- **WHEN** `calculate()` is called with `None` for `tax_year_start`
- **THEN** return `TaxReport` containing all tax years with disposals
- **AND** group disposals into separate `TaxYearSummary` entries by tax period
- **AND** sort `tax_years` chronologically (earliest first)

### Requirement: Corporate Actions

The system SHALL adjust pools for SPLIT, UNSPLIT, and CAPRETURN.

#### Scenario: Split adjustment

- **WHEN** a 2:1 split occurs
- **THEN** double quantity, maintain total cost

#### Scenario: Capital return

- **WHEN** capital return is received
- **THEN** reduce pool cost basis by return amount

### Requirement: Gain/Loss Calculation

The system SHALL calculate gain or loss: proceeds - allowable cost - expenses.

#### Scenario: Calculation

- **WHEN** disposal is matched
- **THEN** compute gain (positive) or loss (negative)

### Requirement: Accumulation Dividends

The system SHALL increase pool cost basis for accumulation dividends.

#### Scenario: Dividend adjustment

- **WHEN** accumulation dividend is received
- **THEN** add dividend amount to pool cost basis

### Requirement: Validation Contract

The system SHALL document that transaction validation should occur before calculation.

#### Scenario: Calculation without prior validation

- **WHEN** `calculate()` is called without prior `validate()` call
- **THEN** calculation proceeds but may encounter unexpected behavior on invalid inputs
- **AND** documentation warns that `validate()` should be called first

#### Scenario: Explicit validated calculation

- **WHEN** caller wants guaranteed validation
- **THEN** caller can call `validate()` first and check `is_valid()` before proceeding
- **AND** this pattern is documented in function doc comments
