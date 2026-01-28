## ADDED Requirements

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
