## MODIFIED Requirements

### Requirement: Report Sections

The system SHALL include Summary, Tax Year Details, Holdings, Transactions, with disposal count, net gain, total gains, total losses, and gross proceeds in the summary.

#### Scenario: Content parity

- **WHEN** generating PDF
- **THEN** include same data as plain text format with professional layout
- **AND** summary table shows disposal count, net gain, total gains, total losses, and gross proceeds (for SA108 Box 21)

## ADDED Requirements

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
