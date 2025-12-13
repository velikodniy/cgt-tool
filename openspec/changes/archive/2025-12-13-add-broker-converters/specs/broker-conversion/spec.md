## ADDED Requirements

### Requirement: Converter Trait

The system SHALL provide a `BrokerConverter` trait that broker modules implement to convert export files to CGT DSL format, with broker-specific input types.

#### Scenario: Implement converter for new broker

- **WHEN** a new broker module implements `BrokerConverter`
- **THEN** it defines an associated `Input` type for broker-specific file requirements
- **AND** it provides `convert(input) -> Result<ConvertOutput, ConvertError>` accepting the input type
- **AND** it provides `broker_name() -> &'static str` for identification

### Requirement: WASM Compatibility

The converter core SHALL operate without filesystem or network IO, accepting file contents as strings.

#### Scenario: Convert in browser environment

- **WHEN** converter is compiled to WASM
- **THEN** it accepts file contents as `String` parameters
- **AND** returns CGT DSL content as `String`
- **AND** does not require filesystem access

### Requirement: Schwab Transactions CSV Parser

The system SHALL parse Charles Schwab transaction CSV exports containing columns: Date, Action, Symbol, Description, Price, Quantity, Fees & Comm, Amount.

#### Scenario: Parse basic buy transaction

- **WHEN** CSV contains row `07/02/2020,Buy,VUAG,VUAG INC,$30.2,30.5,$4,-$925.1`
- **THEN** output `2020-07-02 BUY VUAG 30.5 @ 30.2 USD EXPENSES 4 USD`

#### Scenario: Parse sell transaction

- **WHEN** CSV contains row `06/06/2020,Sell,VUAG,VUAG INC,$33,90,$5,$2965`
- **THEN** output `2020-06-06 SELL VUAG 90 @ 33 USD EXPENSES 5 USD`

#### Scenario: Parse dividend

- **WHEN** CSV contains row `03/04/2021,Cash Dividend,BNDX,VANGUARD...,,,,$3.65`
- **THEN** output `2021-03-04 DIVIDEND BNDX 0 TOTAL 3.65 USD TAX 0 USD`

#### Scenario: Parse dividend with tax withheld

- **WHEN** CSV contains dividend row followed by NRA Withholding row on same date for same symbol
- **THEN** combine into single `DIVIDEND` with `TAX` amount from withholding

#### Scenario: Skip non-CGT transactions

- **WHEN** CSV contains Wire Sent, Wire Received, Credit Interest, or similar cash movements
- **THEN** skip these rows
- **AND** include count of skipped transactions in header comment

### Requirement: Schwab Awards JSON Parser

The system SHALL parse Schwab Equity Awards JSON to obtain Fair Market Value prices for RSU vesting events.

#### Scenario: RSU vesting with FMV lookup

- **WHEN** transactions CSV contains `Stock Plan Activity` for symbol on date
- **AND** awards JSON contains FMV price for that symbol and date
- **THEN** output `BUY` transaction at the FMV price from awards file

#### Scenario: RSU vesting without awards file

- **WHEN** transactions CSV contains `Stock Plan Activity`
- **AND** no awards file is provided
- **THEN** return error indicating awards file is required for Stock Plan Activity

### Requirement: Date Format Handling

The system SHALL parse Schwab date formats including "as of" notation.

#### Scenario: Standard date format

- **WHEN** date is `02/09/2021`
- **THEN** parse as 2021-02-09

#### Scenario: "As of" date format

- **WHEN** date is `02/25/2021 as of 02/21/2021`
- **THEN** use primary date `02/25/2021` (2021-02-25)
- **AND** include "as of" note in output comment

### Requirement: Output Metadata Comments

The system SHALL generate header comments in output containing conversion metadata.

#### Scenario: Standard conversion header

- **WHEN** conversion completes successfully
- **THEN** output begins with comments containing:
  - Source broker name
  - Input file names (if available)
  - Conversion timestamp
  - Count of skipped/unsupported transactions (if any)

#### Scenario: Transaction-level comments

- **WHEN** a transaction has notable characteristics (RSU vesting, "as of" date)
- **THEN** preceding comment explains the context

### Requirement: Chronological Output Order

The system SHALL output transactions in chronological order (oldest first).

#### Scenario: Reorder reversed input

- **WHEN** Schwab CSV is in reverse chronological order (newest first)
- **THEN** output transactions oldest-to-newest

### Requirement: CLI Convert Subcommand

The CLI SHALL provide a `convert` subcommand with broker-specific subcommands, each defining its own arguments.

#### Scenario: Convert Schwab with positional transactions file

- **WHEN** user runs `cgt convert schwab transactions.csv`
- **THEN** convert transactions CSV and output CGT DSL to stdout

#### Scenario: Convert Schwab with awards file

- **WHEN** user runs `cgt convert schwab transactions.csv --awards awards.json`
- **THEN** convert transactions CSV using FMV prices from awards JSON
- **AND** output CGT DSL to stdout

#### Scenario: Convert without awards file when RSUs present

- **WHEN** user runs `cgt convert schwab transactions.csv` (no awards)
- **AND** transactions contain Stock Plan Activity
- **THEN** exit with error explaining awards file is required

#### Scenario: Write to file

- **WHEN** user runs `cgt convert schwab transactions.csv -o output.cgt`
- **THEN** write CGT DSL to specified file

#### Scenario: Show broker-specific help

- **WHEN** user runs `cgt convert schwab --help`
- **THEN** display Schwab-specific arguments (transactions file, --awards option)

### Requirement: Error Reporting

The system SHALL provide clear error messages for conversion failures.

#### Scenario: Invalid CSV format

- **WHEN** CSV is missing required columns
- **THEN** error message lists missing columns

#### Scenario: Unparseable date

- **WHEN** date cannot be parsed
- **THEN** error message includes line number and invalid date value

#### Scenario: Unknown action type

- **WHEN** transaction has unrecognized action
- **THEN** warn and skip (do not fail entire conversion)
