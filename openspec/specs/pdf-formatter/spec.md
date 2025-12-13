# PDF Formatter Specification

## Purpose

Generate professional PDF CGT reports using embedded Typst.

## Requirements

### Requirement: Embedded Generation

The system SHALL generate PDFs using typst-as-lib without external tools.

#### Scenario: Self-contained

- **WHEN** generating PDF
- **THEN** use embedded Typst engine with bundled fonts

### Requirement: Output Path

The system SHALL write PDF to `--output` path or default to input filename with .pdf.

#### Scenario: File output

- **WHEN** `--format pdf` is used
- **THEN** write to specified or default path

### Requirement: Report Sections

The system SHALL include Summary, Tax Year Details, Holdings, Transactions.

#### Scenario: Content parity

- **WHEN** generating PDF
- **THEN** include same data as plain text format with professional layout

### Requirement: Professional Layout

The system SHALL use tables, clear headings, and readable fonts.

#### Scenario: Formatting

- **WHEN** rendering PDF
- **THEN** use formatted tables with column headers and hierarchical headings

### Requirement: UK Formatting

The system SHALL use £ symbol for currency and DD/MM/YYYY for dates.

#### Scenario: Currency display

- **WHEN** displaying amounts
- **THEN** use £ with 2 decimals; show foreign currency symbols for original amounts

### Requirement: Error Handling

The system SHALL return clear errors if generation fails.

#### Scenario: Failure handling

- **WHEN** PDF generation or file write fails
- **THEN** report clear error message (never fail silently)
