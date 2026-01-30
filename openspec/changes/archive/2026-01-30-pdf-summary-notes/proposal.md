## Why

PDF reports currently rely on plain-text footnotes that can be hard to map to specific columns. Superscript references in column headers (e.g. "Disposals¹") with explanatory footnotes will clarify definitions and link them directly to SA108 tax return guidance, making the report more professional and easier to use for tax filing.

## What Changes

- Update PDF summary table headers to include superscript footnote markers (e.g. "Disposals¹", "Proceeds²").
- Replace the current list of notes with numbered footnotes that explain each column using SA108 and HMRC manual references.
- Keep plain text notes as a simple list since superscripts aren't supported there.

## Capabilities

### New Capabilities

- None.

### Modified Capabilities

- `pdf-formatter`: Update summary table headers with superscripts and replace notes block with numbered footnotes.

## Impact

- `crates/cgt-formatter-pdf/src/templates/report.typ`: Typst template update for superscripts and footnotes.
- No changes to calculation logic or plain text output structure (beyond note content alignment if needed).

## Verification

- Generate a PDF report and visually verify that column headers have superscripts and footnotes appear correctly below the table.
- Verify that footnote content accurately reflects HMRC guidance (SA108 Box 20/21, CG51560).
