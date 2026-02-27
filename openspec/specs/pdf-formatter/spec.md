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

The system SHALL include Summary, Tax Year Details, Holdings, Transactions, with disposal count (grouped by same-day aggregation), net gain, total gains, total losses, and gross proceeds in the summary.

#### Scenario: Content parity

- **WHEN** generating PDF
- **THEN** include same data as plain text format with professional layout
- **AND** summary table shows disposal count, net gain, total gains, total losses, and gross proceeds (for SA108 Box 21)

### Requirement: Superscript Footnotes

The system SHALL use superscript markers in the summary table headers and display corresponding numbered footnotes below the table.

#### Scenario: Headers with superscripts

- **WHEN** generating the PDF summary table
- **THEN** the "Disposals" header includes a superscript marker (e.g., ¹)
- **AND** the "Proceeds" header includes a superscript marker (e.g., ²)
- **AND** the "Gains" and "Losses" headers include a superscript marker (e.g., ³)

#### Scenario: Two-line gains headers

- **WHEN** generating the PDF summary table
- **THEN** the gains headers render as two lines ("Gains" on the first line, qualifier on the second line)

#### Scenario: Footnote content

- **WHEN** generating the PDF summary table
- **THEN** a footnote block appears below the table
- **AND** footnote 1 explains grouped disposals (CG51560)
- **AND** footnote 2 explains Gains/Losses as net allowable per SA108
- **AND** footnote 3 defines Proceeds as SA108 Box 21

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

The system SHALL return clear errors if generation fails, and PDF-generation failures SHALL be represented by a PDF formatter-owned error type rather than a `cgt-core` error variant.

#### Scenario: Failure handling

- **WHEN** PDF generation or file write fails
- **THEN** report clear error message (never fail silently)

#### Scenario: Error ownership boundary

- **WHEN** Typst compilation, Decimal-to-float conversion, or PDF export fails
- **THEN** return an error variant owned by the PDF formatter crate
- **AND** do not construct a `cgt-core::CgtError` variant for PDF-specific failures

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

### Requirement: Use Core Computed Methods

The PDF formatter SHALL obtain disposal and tax-year derived totals from `cgt-core` model methods rather than re-implementing formulas locally.

#### Scenario: PDF summary and disposal rendering

- **WHEN** PDF output renders disposal totals and annual summary values
- **THEN** the formatter uses `Disposal` and `TaxYearSummary` computed methods for those values
- **AND** output values remain unchanged from prior behavior

### Requirement: Shared Date and Ticker Sort Usage

The PDF formatter MUST use the canonical date+ticker ordering helper from `cgt-core` for any formatter-level sorting that follows date ascending and ticker ascending semantics.

#### Scenario: Formatter call sites use shared helper

- **WHEN** PDF formatter sorts rows or records by date then ticker
- **THEN** it calls the shared comparator helper instead of defining a local inline comparator
- **AND** visible report ordering is unchanged

#### Scenario: Cross-format ordering parity

- **WHEN** the same report data is rendered in plain text and PDF
- **THEN** both outputs use the same date+ticker ordering behavior
