## MODIFIED Requirements

### Requirement: Report Sections

The system SHALL include Summary, Tax Year Details, Holdings, Transactions, with gross proceeds in the summary.

#### Scenario: Content parity

- **WHEN** generating PDF
- **THEN** include same data as plain text format with professional layout
- **AND** summary table shows gross proceeds (for SA108 Box 21)

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

## ADDED Requirements

### Requirement: SA108 Guidance Note

The system SHALL include a brief note in the PDF report explaining which values correspond to SA108 boxes.

#### Scenario: Summary section note

- **WHEN** generating a PDF report with disposals
- **THEN** include a note indicating that Proceeds = SA108 Box 21, and that allowable costs include all fees
