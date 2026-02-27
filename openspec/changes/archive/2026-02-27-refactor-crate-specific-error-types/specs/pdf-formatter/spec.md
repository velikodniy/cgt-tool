## MODIFIED Requirements

### Requirement: Error Handling

The system SHALL return clear errors if generation fails, and PDF-generation failures SHALL be represented by a PDF formatter-owned error type rather than a `cgt-core` error variant.

#### Scenario: Failure handling

- **WHEN** PDF generation or file write fails
- **THEN** report clear error message (never fail silently)

#### Scenario: Error ownership boundary

- **WHEN** Typst compilation, Decimal-to-float conversion, or PDF export fails
- **THEN** return an error variant owned by the PDF formatter crate
- **AND** do not construct a `cgt-core::CgtError` variant for PDF-specific failures
