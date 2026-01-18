## MODIFIED Requirements

### Requirement: CLI Convert Subcommand

The CLI SHALL provide a `convert` subcommand with broker-specific subcommands, each defining its own arguments.

#### Scenario: Convert Schwab with positional transactions file

- **WHEN** user runs `cgt convert schwab transactions.json`
- **THEN** convert transactions JSON and output CGT DSL to stdout

#### Scenario: Convert Schwab with awards file

- **WHEN** user runs `cgt convert schwab transactions.json --awards awards.json`
- **THEN** convert transactions JSON using FMV prices from awards JSON file
- **AND** output CGT DSL to stdout

#### Scenario: Convert without awards file when RSUs present

- **WHEN** user runs `cgt convert schwab transactions.json` (no awards)
- **AND** transactions contain Stock Plan Activity
- **THEN** exit with error explaining awards file is required for RSU vesting

#### Scenario: Write to file

- **WHEN** user runs `cgt convert schwab transactions.json -o output.cgt`
- **THEN** write CGT DSL to specified file

#### Scenario: Show broker-specific help

- **WHEN** user runs `cgt convert schwab --help`
- **THEN** display Schwab-specific arguments (transactions file, --awards option)
