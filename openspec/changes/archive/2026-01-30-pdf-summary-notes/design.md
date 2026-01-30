## Context

Currently, the PDF summary table uses generic headers and a detached list of notes at the bottom. To make the report professional and directly useful for filling out SA108 forms, we want to link specific columns (Disposals, Proceeds, Gains/Losses) to their definitions and tax return boxes using standard footnote conventions (superscripts). Plain text reports do not support rich formatting, so they will retain the list format.

## Goals / Non-Goals

**Goals:**

- Add superscript markers to PDF summary table headers.
- Replace the note list in PDF with a numbered footnote section.
- Ensure footnotes explicitly reference HMRC guidance (CG51560) and SA108 form boxes.

**Non-Goals:**

- Changing the layout or columns of the table itself.
- Implementing superscripts in plain text or JSON output.

## Decisions

1. **Use Typst's `#super[]` in headers.**

   - **Why:** Typst supports rich text in table headers, and `#super("1")` avoids reliance on unicode glyph availability.
   - **Decision:** Use `#super("1")`, `#super("2")`, `#super("3")` in the Typst template strings.

2. **Numbering Scheme:**

   - ¹ **Disposals**: Clarify same-day grouping vs raw counts (CG51560).
   - ² **Gains/Losses**: Link to SA108 Box 23/24 (net allowable).
   - ³ **Proceeds**: Link to SA108 Box 21 (gross value).

3. **Footnote Layout:**

   - **Why:** A simple numbered list below the table matches the existing "Notes" block style but with explicit numbers matching the headers.

## Risks / Trade-offs

- **[Font support]** → `#super[]` avoids missing unicode superscript glyphs in embedded fonts.
- **[Header Width]** → Adding markers might slightly increase header width; column widths were recently tuned and should accommodate a single character.

## Migration Plan

- Update `report.typ` template. No data migration needed.
