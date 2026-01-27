# cli-output Specification

## Purpose

TBD - created by archiving change fix-cli-output-flag. Update Purpose after archive.

## Requirements

### Requirement: Report Output Destination

The CLI MUST respect the output destination specified by the user for all report formats.

#### Scenario: JSON output to file

- **WHEN** running `report` with `--format json` AND `--output file.json`
- **THEN** the JSON report should be written to `file.json`
- **AND** nothing should be printed to stdout

#### Scenario: Plain output to file

- **WHEN** running `report` with `--format plain` AND `--output file.txt`
- **THEN** the plain text report should be written to `file.txt`
- **AND** nothing should be printed to stdout

#### Scenario: Default to stdout

- **WHEN** running `report` WITHOUT `--output`
- **THEN** the report should be printed to stdout
