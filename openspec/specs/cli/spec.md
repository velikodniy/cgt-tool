# CLI Specification

## Purpose

Command-line interface for parsing transactions and generating CGT reports.

## Requirements

### Requirement: Parse Command

The system SHALL provide `parse` to validate and output transactions as JSON.

#### Scenario: Parse single file

- **WHEN** `cgt-tool parse file.cgt` is run
- **THEN** output parsed transactions as JSON to stdout
- **AND** output errors with line numbers to stderr if invalid

#### Scenario: Parse multiple files

- **WHEN** `cgt-tool parse file1.cgt file2.cgt file3.cgt` is run
- **THEN** concatenate file contents in argument order
- **AND** parse the combined content
- **AND** output parsed transactions as JSON to stdout
- **AND** prefix error line numbers with filename if invalid

### Requirement: Report Command

The system SHALL provide `report` to generate CGT reports.

#### Scenario: Generate report from single file

- **WHEN** `cgt-tool report file.cgt --year 2024` is run
- **THEN** generate report for tax year 2024/25

#### Scenario: Generate report from multiple files

- **WHEN** `cgt-tool report file1.cgt file2.cgt --year 2024` is run
- **THEN** concatenate file contents in argument order
- **AND** parse the combined content
- **AND** generate report for tax year 2024/25

### Requirement: Format Selection

The system SHALL support `--format` with values: plain (default), json, pdf.

#### Scenario: Format options

- **WHEN** `--format plain|json|pdf` is specified
- **THEN** generate report in requested format
- **AND** report error for unsupported format values

### Requirement: Output Path

The system SHALL support `--output` for PDF destination.

#### Scenario: PDF output from single file

- **WHEN** `--format pdf` with single input file and no `--output`
- **THEN** default output filename is input filename with .pdf extension

#### Scenario: PDF output from multiple files

- **WHEN** `--format pdf` with multiple input files and no `--output`
- **THEN** default output filename is `report.pdf`

#### Scenario: PDF output with explicit path

- **WHEN** `--format pdf --output path.pdf` is specified
- **THEN** write to specified path

#### Scenario: PDF output refuses to overwrite

- **WHEN** `--format pdf` and default output file already exists
- **THEN** error with message indicating file exists
- **AND** suggest using `--output` to specify a different path

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
