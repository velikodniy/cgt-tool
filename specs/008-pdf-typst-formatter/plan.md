# Implementation Plan: PDF Typst Formatter

**Branch**: `008-pdf-typst-formatter` | **Date**: 2025-12-09 | **Spec**: [spec.md](./spec.md)
**Input**: Feature specification from `/specs/008-pdf-typst-formatter/spec.md`

## Summary

Implement a PDF formatter for CGT tax reports using embedded Typst via the `typst-as-lib` crate. The formatter will generate professional, printable PDF documents containing tax summaries, disposal details, holdings, and transaction history. No external tool installation required - all PDF generation happens within the Rust binary using embedded fonts.

## Technical Context

**Language/Version**: Rust 2024 edition (matching existing crates)
**Primary Dependencies**: typst-as-lib (v0.15+), typst-pdf, typst-assets (for embedded fonts)
**Storage**: N/A (generates PDF files to filesystem)
**Testing**: cargo test (integration tests comparing PDF generation success)
**Target Platform**: macOS, Linux, Windows (same as existing CLI)
**Project Type**: Rust workspace with multiple crates
**Performance Goals**: PDF generation under 5 seconds for 1000 transactions
**Constraints**: No external tool dependencies, must use embedded Typst
**Scale/Scope**: Single tax year reports, up to 1000 transactions typical

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

| Principle                         | Status  | Notes                                                                                                  |
| --------------------------------- | ------- | ------------------------------------------------------------------------------------------------------ |
| I. Deep Modules & Simplicity      | ✅ PASS | New crate `cgt-formatter-pdf` follows existing pattern, simple interface `format() -> Result<Vec<u8>>` |
| II. Safety & Robustness           | ✅ PASS | Proper error handling with `Result<T, CgtError>`, no panics in production code                         |
| III. Modern Testing Standards     | ✅ PASS | Integration tests will verify PDF generation for all 26 test cases                                     |
| IV. User Experience Consistency   | ✅ PASS | CLI interface consistent with existing `--format plain/json` pattern                                   |
| V. Performance & Efficiency       | ✅ PASS | Embedded fonts avoid filesystem lookups, Typst is fast                                                 |
| VI. Domain Mastery & Verification | ✅ PASS | Reuses existing TaxReport structure, numerical values match plain formatter                            |

## Project Structure

### Documentation (this feature)

```text
specs/008-pdf-typst-formatter/
├── plan.md              # This file
├── research.md          # Phase 0 output
├── data-model.md        # Phase 1 output
├── quickstart.md        # Phase 1 output
└── tasks.md             # Phase 2 output
```

### Source Code (repository root)

```text
crates/
├── cgt-core/            # Existing - TaxReport, models, exemption
├── cgt-formatter-plain/ # Existing - plain text formatter
├── cgt-formatter-pdf/   # NEW - PDF formatter using Typst
│   ├── Cargo.toml
│   ├── src/
│   │   ├── lib.rs       # Public API: format(report, transactions) -> Result<Vec<u8>>
│   │   └── template.typ # Embedded Typst template
│   └── fonts/           # Embedded fonts (optional, use typst-assets)
└── cgt-cli/             # Existing - add --format pdf support

tests/
├── inputs/              # Existing .cgt test files
├── json/                # Existing JSON expected outputs
└── plain/               # Existing plain text expected outputs
```

**Structure Decision**: Follow existing workspace pattern. New `cgt-formatter-pdf` crate parallels `cgt-formatter-plain`. Template embedded as string constant or include_str!().

## Complexity Tracking

No constitution violations - design follows established patterns.
