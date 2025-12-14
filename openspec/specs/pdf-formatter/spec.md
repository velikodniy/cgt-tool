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

### Requirement: Proceeds Breakdown Parity

The PDF disposal calculation SHALL display net proceeds using quantity × unit price minus sale expenses, matching the calculated proceeds and staying aligned with the plain text formatter.

#### Scenario: Sell with sale expenses

- **WHEN** a disposal includes sale expenses
- **THEN** the proceeds line is shown as `<quantity> × £<unit price> - £<sale expenses> = £<net proceeds>`
- **AND** `<net proceeds>` equals `(unit price × quantity) - sale expenses` rounded using the currency’s minor units (per the currency policy).

#### Scenario: Sell without sale expenses

- **WHEN** sale expenses are zero
- **THEN** the proceeds line is shown as `<quantity> × £<unit price> = £<net proceeds>`
- **AND** `<net proceeds>` equals `unit price × quantity` rounded using the currency’s minor units (per the currency policy).
- **AND** the proceeds line omits the `- £0` term when sale expenses are zero.
