## MODIFIED Requirements

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
