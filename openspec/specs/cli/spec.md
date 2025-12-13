# CLI Specification

## Purpose

Command-line interface for parsing transactions and generating CGT reports.

## Requirements

### Requirement: Parse Command

The system SHALL provide `parse` to validate and output transactions as JSON.

#### Scenario: Parse file

- **WHEN** `cgt-cli parse file.cgt` is run
- **THEN** output parsed transactions as JSON to stdout
- **AND** output errors with line numbers to stderr if invalid

### Requirement: Report Command

The system SHALL provide `report` to generate CGT reports.

#### Scenario: Generate report

- **WHEN** `cgt-cli report file.cgt --year 2024` is run
- **THEN** generate report for tax year 2024/25

### Requirement: Format Selection

The system SHALL support `--format` with values: plain (default), json, pdf.

#### Scenario: Format options

- **WHEN** `--format plain|json|pdf` is specified
- **THEN** generate report in requested format
- **AND** report error for unsupported format values

### Requirement: Output Path

The system SHALL support `--output` for PDF destination.

#### Scenario: PDF output

- **WHEN** `--format pdf` with optional `--output path.pdf`
- **THEN** write to specified path or default to input filename with .pdf

### Requirement: FX Folder

The system SHALL accept `--fx-folder` for custom exchange rate files.

#### Scenario: Custom rates

- **WHEN** `--fx-folder ./rates` is specified
- **THEN** load rates from folder, falling back to bundled rates

### Requirement: Actionable Errors

The system SHALL output clear, actionable error messages.

#### Scenario: Error output

- **WHEN** any operation fails
- **THEN** show context (line number, value, expected format) and suggested fix
