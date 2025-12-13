## MODIFIED Requirements

### Requirement: Schwab Awards Parser

The system SHALL parse Schwab Equity Awards in both JSON and CSV formats to obtain Fair Market Value prices for RSU vesting events, auto-detecting the format based on file extension.

#### Scenario: Parse JSON awards format

- **WHEN** awards file has `.json` extension
- **THEN** parse as JSON with structure: `{"EquityAwards": [{"Symbol": "...", "EventDate": "MM/DD/YYYY", "FairMarketValuePrice": "$..."}]}`
- **AND** extract Symbol, EventDate, and FairMarketValuePrice from each award

#### Scenario: Parse CSV awards format

- **WHEN** awards file has `.csv` extension
- **THEN** parse as CSV with paired-row structure
- **AND** extract Symbol from transaction row
- **AND** extract AwardDate and FairMarketValuePrice from following award row

#### Scenario: CSV paired-row structure

- **WHEN** CSV contains transaction row with Action="Lapse"
- **AND** next row has empty transaction fields but filled AwardDate and FairMarketValuePrice
- **THEN** combine Symbol from transaction row with AwardDate and FairMarketValuePrice from award row

#### Scenario: RSU vesting with CSV awards

- **WHEN** transactions CSV contains `Stock Plan Activity` for symbol on date
- **AND** awards CSV contains FMV price for that symbol and date
- **THEN** output `BUY` transaction at the FMV price from awards file

#### Scenario: RSU vesting without awards file

- **WHEN** transactions CSV contains `Stock Plan Activity`
- **AND** no awards file is provided
- **THEN** return error indicating awards file is required for Stock Plan Activity

#### Scenario: Default to JSON when no extension

- **WHEN** awards file has no extension or unrecognized extension
- **THEN** attempt to parse as JSON for backward compatibility
- **AND** return appropriate error if JSON parsing fails
