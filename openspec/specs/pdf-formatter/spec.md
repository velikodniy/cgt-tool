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

The system SHALL include Summary, Tax Year Details, Holdings, Transactions, with gross proceeds in the summary.

#### Scenario: Content parity

- **WHEN** generating PDF
- **THEN** include same data as plain text format with professional layout
- **AND** summary table shows gross proceeds (for SA108 Box 21)

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

The PDF disposal calculation SHALL display both gross and net proceeds with clear labels, matching the plain text formatter and staying aligned with SA108 requirements.

#### Scenario: Sell with sale expenses

- **WHEN** a disposal includes sale expenses
- **THEN** the proceeds section shows gross proceeds as `<quantity> × £<unit price> = £<gross proceeds>`
- **AND** shows net proceeds as `£<gross proceeds> - £<sale expenses> fees = £<net proceeds>`
- **AND** `<net proceeds>` equals `(unit price × quantity) - sale expenses`

#### Scenario: Sell without sale expenses

- **WHEN** sale expenses are zero
- **THEN** the proceeds section shows `<quantity> × £<unit price> = £<gross proceeds>`
- **AND** gross and net proceeds are equal
- **AND** the fees line is omitted

#### Scenario: Same-day merge with multiple sells

- **WHEN** multiple sells occur on the same day for the same ticker
- **THEN** display the weighted average unit price calculated as total_gross_proceeds / total_quantity
- **AND** the displayed price reflects the actual average, not an arbitrary individual sell price

### Requirement: Shared Formatter Dependency

The system SHALL use `cgt-format` for all currency formatting instead of implementing local helpers.

#### Scenario: Currency formatting source

- **WHEN** formatting currency values in PDF output
- **THEN** use `CurrencyFormatter` from `cgt-format`
- **AND** do not implement ad-hoc formatting helpers

### Requirement: Formatting Parity

The system SHALL produce identical currency/date formatting as the plain text formatter.

#### Scenario: Consistent output across formats

- **WHEN** formatting the same value in PDF and plain text
- **THEN** the formatted string is identical
- **AND** both use the same `CurrencyFormatter` implementation

### Requirement: SA108 Guidance Note

The system SHALL include a brief note in the PDF report explaining which values correspond to SA108 boxes.

#### Scenario: Summary section note

- **WHEN** generating a PDF report with disposals
- **THEN** include a note indicating that Proceeds = SA108 Box 21, and that allowable costs include all fees
