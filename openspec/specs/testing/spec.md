# testing Specification

## Purpose

TBD - created by archiving change improve-test-infrastructure. Update Purpose after archive.

## Requirements

### Requirement: Test File Organization

The system SHALL organize tests in separate files under each crate's `tests/` directory.

#### Scenario: Crate test structure

- **WHEN** a crate has tests
- **THEN** tests reside in `crates/<crate>/tests/<module>_tests.rs`
- **AND** source files in `src/` do not contain `#[cfg(test)]` modules

#### Scenario: Integration tests access

- **WHEN** tests need access to public API
- **THEN** tests import the crate as an external dependency
- **AND** tests can access all `pub` items

### Requirement: Test Coverage Measurement

The system SHALL support measuring test coverage using `cargo-llvm-cov`.

#### Scenario: Coverage report generation

- **WHEN** developer runs `cargo llvm-cov --html`
- **THEN** an HTML coverage report is generated in `target/llvm-cov/html/`
- **AND** the report shows line coverage percentages per file

#### Scenario: CI coverage reporting

- **WHEN** CI runs coverage job
- **THEN** coverage data is generated in lcov format
- **AND** coverage percentage is available for tracking

### Requirement: Edge Case Test Coverage

The system SHALL include tests for edge cases identified from HMRC guidance and forums.

#### Scenario: Multi-currency same-day

- **WHEN** shares are bought in USD and sold in GBP on the same day
- **THEN** same-day matching applies
- **AND** both transactions are converted to GBP using that day's FX rate

#### Scenario: Bed and breakfast boundary

- **WHEN** shares are sold on day D
- **AND** shares are repurchased on day D+30
- **THEN** B&B rule applies (within 30 days)

#### Scenario: Bed and breakfast outside boundary

- **WHEN** shares are sold on day D
- **AND** shares are repurchased on day D+31
- **THEN** B&B rule does NOT apply
- **AND** sale matches against Section 104 pool

#### Scenario: Partial B&B with S104 fallback

- **WHEN** 100 shares are sold
- **AND** 40 shares are repurchased within 30 days
- **THEN** 40 shares match via B&B
- **AND** 60 shares match against Section 104 pool

#### Scenario: Same-day buy-sell-buy

- **WHEN** shares are bought, sold, and bought again on the same day
- **THEN** all same-day transactions are aggregated
- **AND** net position determines matching

#### Scenario: Capital return exceeds cost basis

- **WHEN** capital return amount exceeds the cost basis
- **THEN** the excess creates a gain
- **AND** cost basis is reduced to zero

#### Scenario: Split then immediate sell

- **WHEN** a stock split occurs
- **AND** shares are sold on the same day
- **THEN** split is applied before matching
- **AND** disposal quantity reflects post-split shares

### Requirement: Cross-Validation

The system SHALL provide scripts to validate calculations against external UK CGT calculators.

#### Scenario: KapJI calculator comparison

- **WHEN** `scripts/cross-validate.sh` is run with a .cgt file
- **THEN** results are compared against KapJI/capital-gains-calculator
- **AND** discrepancies greater than £1 are reported

#### Scenario: cgtcalc comparison

- **WHEN** `scripts/cross-validate.sh` is run with a .cgt file
- **THEN** results are compared against mattjgalloway/cgtcalc
- **AND** discrepancies greater than £1 are reported

#### Scenario: Format conversion

- **WHEN** converting from .cgt to external calculator format
- **THEN** dates are converted (YYYY-MM-DD to DD/MM/YYYY)
- **AND** transaction types are mapped correctly
- **AND** currency is handled appropriately

### Requirement: Complex Multi-Year Test Fixture

The system SHALL include a realistic multi-year test fixture for comprehensive validation.

#### Scenario: Multi-year fixture properties

- **WHEN** the RealisticMultiYear fixture is processed
- **THEN** it spans 2-3 UK tax years
- **AND** includes multiple tickers (3+)
- **AND** includes same-day, B&B, and S104 matches
- **AND** includes corporate actions (splits, dividends, capital returns)

#### Scenario: Multi-year fixture verification

- **WHEN** the RealisticMultiYear fixture is processed
- **THEN** results match expected JSON output
- **AND** results match expected plain text output
- **AND** results are consistent with external calculator output

### Requirement: Test Documentation

Tests SHALL include documentation explaining the scenario being tested.

#### Scenario: Fixture documentation

- **WHEN** a .cgt test fixture is created
- **THEN** it includes header comments describing:
  - Test purpose
  - Rules being tested
  - Expected outcome
  - Verification status

#### Scenario: Unit test documentation

- **WHEN** a unit test is created for an edge case
- **THEN** it includes comments explaining the HMRC rule being tested
- **AND** references relevant HMRC guidance (e.g., CG51560)

### Requirement: Comprehensive Synthetic Test Fixture

The system SHALL include a synthetic test fixture (`SyntheticComplex.cgt`) that exercises all CGT matching rules and edge cases across multiple tax years.

#### Scenario: Multi-year coverage

- **WHEN** the SyntheticComplex fixture is processed
- **THEN** it spans 5 UK tax years (2020/21 through 2024/25)
- **AND** includes transactions in each tax year

#### Scenario: Multi-ticker coverage

- **WHEN** the SyntheticComplex fixture is processed
- **THEN** it includes at least 3 tickers: ACME (USD), BETA (USD), GAMA (GBP)
- **AND** each ticker exercises different matching scenarios

#### Scenario: Same Day matching

- **WHEN** the SyntheticComplex fixture is processed
- **THEN** it includes same-day buy and sell transactions
- **AND** includes multiple buys and sells on the same day (aggregation)
- **AND** includes buy-sell-buy patterns on the same day

#### Scenario: Bed and Breakfast matching

- **WHEN** the SyntheticComplex fixture is processed
- **THEN** it includes sell followed by repurchase within 30 days
- **AND** includes partial B&B (sell 100, buy back 60 within 30 days)
- **AND** includes exact 30-day boundary case (D+30 = matches, D+31 = no match)

#### Scenario: Section 104 pool

- **WHEN** the SyntheticComplex fixture is processed
- **THEN** it includes regular trading that builds and depletes S104 pools
- **AND** pool cost basis is correctly tracked across tax years

#### Scenario: Corporate actions

- **WHEN** the SyntheticComplex fixture is processed
- **THEN** it includes at least one stock split (SPLIT operation)
- **AND** it includes at least one capital return (CAPRETURN operation)

#### Scenario: Multi-currency

- **WHEN** the SyntheticComplex fixture is processed
- **THEN** it includes GBP-denominated transactions (GAMA ticker)
- **AND** it includes USD-denominated transactions requiring FX conversion

#### Scenario: Tax year boundary

- **WHEN** the SyntheticComplex fixture is processed
- **THEN** it includes transactions near tax year boundaries (April 5/6)
- **AND** gains are correctly attributed to the appropriate tax year

#### Scenario: Golden file verification

- **WHEN** the SyntheticComplex fixture is processed
- **THEN** JSON output matches `tests/json/SyntheticComplex.json`
- **AND** plain text output matches `tests/plain/SyntheticComplex.txt`

### Requirement: Schwab JSON Test Fixtures

The system SHALL include Schwab-format JSON test fixtures that can be converted to CGT DSL.

#### Scenario: Synthetic awards file

- **WHEN** `tests/schwab/synthetic-awards.json` is parsed
- **THEN** it follows the exact Schwab awards JSON structure
- **AND** contains Lapse (vest) events with FairMarketValuePrice and SalePrice
- **AND** includes multiple awards vesting on the same date
- **AND** includes AwardId, AwardDate, SharesSoldWithheldForTaxes, NetSharesDeposited, and Taxes fields

#### Scenario: Synthetic transactions file

- **WHEN** `tests/schwab/synthetic-transactions.json` is parsed
- **THEN** it follows the exact Schwab transactions JSON structure
- **AND** contains Stock Plan Activity (RSU settlement)
- **AND** contains Sell transactions with sell-to-cover patterns
- **AND** contains Journal entries for tax withholding
- **AND** includes Date, Action, Symbol, Description, Quantity, Price, Fees & Comm, and Amount fields

#### Scenario: Converter round-trip

- **WHEN** synthetic Schwab JSON files are converted via `cgt convert schwab`
- **THEN** the resulting CGT DSL produces equivalent calculations to `SyntheticComplex.cgt`
- **AND** RSU acquisitions use vest date (from awards) not settlement date

### Requirement: RSU Vesting Edge Cases

The system SHALL include RSU vesting patterns that exercise converter edge cases.

#### Scenario: Multi-award same-day vesting

- **WHEN** the synthetic data includes a vest date
- **THEN** multiple awards (4+) vest on the same date
- **AND** each has different quantities and FMV prices

#### Scenario: Same-day vest and sell-to-cover

- **WHEN** RSUs vest on day D
- **THEN** sell-to-cover transactions occur on day D or D+1
- **AND** Same Day matching applies to vest-date acquisitions sold on vest date

#### Scenario: FMV vs sale price

- **WHEN** RSU sell-to-cover executes
- **THEN** FairMarketValuePrice differs from SalePrice
- **AND** acquisition cost basis uses FMV, not sale price

#### Scenario: Multiple sells at different prices

- **WHEN** sell-to-cover for multiple awards executes
- **THEN** individual sell lots may have different prices
- **AND** each sell is recorded separately in transactions
