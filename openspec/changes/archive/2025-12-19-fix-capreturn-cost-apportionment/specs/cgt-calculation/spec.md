## MODIFIED Requirements

### Requirement: Corporate Actions

The system SHALL adjust pools for SPLIT, UNSPLIT, and CAPRETURN.

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
