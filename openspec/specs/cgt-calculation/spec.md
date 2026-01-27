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

The system SHALL match disposals with acquisitions within 30 days after sale, subject to Same Day matching priority per TCGA92/S106A(9).

When evaluating a potential B&B acquisition date, the system SHALL reserve shares needed for Same Day matching on that date before allowing earlier disposals to consume them. The available quantity for B&B matching SHALL be: `acquisition_quantity - min(same_day_disposal_quantity, acquisition_quantity)`.

#### Scenario: B&B match

- **WHEN** shares are repurchased within 30 days after sale
- **THEN** match chronologically (earliest purchase first)
- **AND** skip if repurchase is beyond 30 days

#### Scenario: Same Day reservation priority

- **WHEN** disposal D1 on Day 1 could B&B match to acquisition A2 on Day 2
- **AND** disposal D2 on Day 2 could Same Day match to acquisition A2
- **AND** A2 quantity is less than D1 + D2 combined
- **THEN** D2's Same Day claim SHALL be satisfied first
- **AND** D1's B&B match SHALL only use remaining shares after Same Day reservation

#### Scenario: Same Day reservation with sufficient shares (data-driven)

- **WHEN** `tests/inputs/SameDayReservation.cgt` is reported to JSON
- **THEN** `tests/json/SameDayReservation.json` shows Same Day matching fully satisfied before B&B from earlier disposals
- **AND** earlier disposals' B&B matches use only shares remaining after Same Day reservation

#### Scenario: Same Day disposal exceeds acquisition

- **WHEN** same-day disposals on an acquisition date exceed the acquisition quantity
- **THEN** the reservation SHALL be capped at the acquisition quantity
- **AND** excess same-day disposal shares proceed to B&B or S104 matching per normal rules

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

### Requirement: Tax Year Total Gain/Loss Aggregation

The system SHALL compute tax-year `total_gain` and `total_loss` from the net result of each disposal after applying CG51560 matching rules and allowable expenditure per TCGA92/S38 (CG15150, CG15250). A disposal SHALL contribute either to `total_gain` (net >= 0) or to `total_loss` (net < 0); `net_gain` SHALL equal `total_gain - total_loss`.

#### Scenario: Mixed rule disposal netting (data-driven)

- **WHEN** `tests/inputs/NetDisposalTotalsMixed.cgt` is reported to JSON
- **THEN** `tests/json/NetDisposalTotalsMixed.json` reports `total_gain` and `total_loss` based on net per-disposal results
- **AND** a disposal with both positive and negative match legs contributes only its net result to totals

#### Scenario: Net zero disposal does not inflate totals

- **WHEN** a disposal has match legs that sum to zero
- **THEN** it contributes to neither `total_gain` nor `total_loss`

#### Scenario: Multi-currency totals use converted GBP

- **WHEN** a disposal includes non-GBP transactions that are converted to GBP during matching
- **THEN** `total_gain` and `total_loss` are aggregated from the GBP proceeds and allowable costs used in disposal matches

### Requirement: Corporate Actions

The system SHALL adjust pools for SPLIT, UNSPLIT, and CAPRETURN, correctly determining share quantities using CGT matching rules (Same Day → B&B → S104).

#### Scenario: Split adjustment

- **WHEN** a 2:1 split occurs
- **THEN** double quantity, maintain total cost

#### Scenario: Capital return

- **WHEN** capital return is received
- **THEN** reduce pool cost basis by return amount

#### Scenario: Capital return cost apportionment

- **WHEN** a CAPRETURN event occurs affecting N shares
- **AND** total holdings across all acquisition lots is M shares (where M >= N)
- **THEN** the cost reduction SHALL be apportioned to each lot based on its proportion of total holdings
- **AND** each lot receives: `adjustment × (lot_shares / total_holdings)`
- **AND** the sum of all lot adjustments equals the total adjustment amount

#### Scenario: Capital return with multiple lots and prior sales

- **WHEN** shares were acquired in multiple lots
- **AND** some shares were sold before the capital return
- **AND** remaining shares span multiple lots
- **THEN** the cost reduction is distributed proportionally across remaining shares in each lot
- **AND** lots acquired after the event date are not affected

#### Scenario: Asset event after same-day buy and sell

- **WHEN** an asset event (CAPRETURN or DIVIDEND) occurs after a date with both BUY and SELL transactions for the same ticker
- **THEN** remaining shares SHALL be calculated using CGT matching rules per CG51560
- **AND** same-day acquisitions are matched first (TCGA92/S105(1))
- **AND** then B&B acquisitions within 30 days after the sale (TCGA92/S106A)
- **AND** then earlier acquisitions (S104 pool)

### Requirement: Gain/Loss Calculation

The system SHALL calculate gain or loss as: net proceeds - allowable cost, where net proceeds = gross proceeds - sale expenses.

#### Scenario: Calculation

- **WHEN** disposal is matched
- **THEN** compute gain (positive) or loss (negative)
- **AND** the Disposal struct contains both gross_proceeds and proceeds (net) fields

#### Scenario: Disposal data structure

- **WHEN** a disposal is created
- **THEN** the Disposal struct contains:
  - `gross_proceeds`: The total sale amount before fees (quantity × unit price)
  - `proceeds`: The net sale amount after fees (gross_proceeds - sale fees)
- **AND** both values are available to formatters for display

### Requirement: Accumulation Dividends

The system SHALL increase pool cost basis for accumulation dividends.

#### Scenario: Dividend adjustment

- **WHEN** accumulation dividend is received
- **THEN** add dividend amount to pool cost basis

#### Scenario: Dividend cost apportionment

- **WHEN** a DIVIDEND event occurs affecting N shares
- **AND** total holdings across all acquisition lots is M shares
- **THEN** the cost increase SHALL be apportioned to each lot based on its proportion of total holdings
- **AND** each lot receives: `adjustment × (lot_shares / total_holdings)`
- **AND** the sum of all lot adjustments equals the total adjustment amount

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
