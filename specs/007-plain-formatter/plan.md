# Implementation Plan: Plain Text Report Formatter

**Branch**: `007-plain-formatter` | **Date**: 2025-12-08 | **Spec**: [spec.md](./spec.md)
**Input**: Feature specification from `/specs/007-plain-formatter/spec.md`

## Summary

Add an extensible plain text formatter for CGT reports matching the cgtcalc output format. The formatter will be a separate crate (`cgt-formatter-plain`) that converts `TaxReport` to human-readable text with SUMMARY, TAX YEAR DETAILS, TAX RETURN INFORMATION, HOLDINGS, TRANSACTIONS, and ASSET EVENTS sections. The CLI will gain a `--format` argument to switch between "plain" (default) and "json" output.

## Technical Context

**Language/Version**: Rust 2024 edition (existing)
**Primary Dependencies**: cgt-core (existing), clap (CLI), rust_decimal, chrono
**Storage**: N/A (output only)
**Testing**: cargo test (unit tests in formatter crate, integration tests for CLI)
**Target Platform**: CLI (macOS/Linux/Windows)
**Project Type**: Rust workspace with multiple crates
**Performance Goals**: Instant formatting (\<100ms for typical portfolios)
**Constraints**: Must match cgtcalc output format exactly for numerical values
**Scale/Scope**: Single tax year report per invocation

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

| Principle                         | Status  | Notes                                                                                          |
| --------------------------------- | ------- | ---------------------------------------------------------------------------------------------- |
| I. Deep Modules & Simplicity      | ✅ PASS | Formatter provides simple `format(TaxReport) -> String` interface hiding formatting complexity |
| II. Safety & Robustness           | ✅ PASS | Uses existing safe data model, no unsafe operations needed                                     |
| III. Modern Testing Standards     | ✅ PASS | Tests will compare output against known cgtcalc format; existing tests preserved               |
| IV. User Experience Consistency   | ✅ PASS | CLI gains intuitive `--format` flag with clear options                                         |
| V. Performance & Efficiency       | ✅ PASS | String formatting is inherently fast for this scale                                            |
| VI. Domain Mastery & Verification | ✅ PASS | Format verified against cgtcalc reference implementation                                       |

**Gate Status**: ✅ PASS - No violations

## Project Structure

### Documentation (this feature)

```text
specs/007-plain-formatter/
├── plan.md              # This file
├── research.md          # Phase 0 output
├── data-model.md        # Phase 1 output
├── quickstart.md        # Phase 1 output
└── tasks.md             # Phase 2 output (/speckit.tasks)
```

### Source Code (repository root)

```text
crates/
├── cgt-core/            # Existing - TaxReport model, calculator
│   └── src/
│       ├── lib.rs
│       ├── models.rs    # TaxReport, Disposal, Match, etc.
│       ├── calculator.rs
│       └── parser.rs
├── cgt-formatter-plain/ # NEW - Plain text formatter crate
│   ├── Cargo.toml
│   └── src/
│       └── lib.rs       # format(TaxReport, &[Transaction]) -> String
└── cgt-cli/             # Existing - CLI with format selection
    ├── Cargo.toml       # Add dependency on cgt-formatter-plain
    └── src/
        ├── main.rs      # Add format dispatching
        └── commands.rs  # Add --format argument

tests/
└── data/
    ├── *.cgt            # Existing input files
    ├── *.json           # Existing expected JSON outputs
    └── *.txt            # NEW expected plain text outputs
```

**Structure Decision**: Add new `cgt-formatter-plain` crate to existing workspace. This follows the extensible architecture requirement (FR-009) while keeping JSON formatting in CLI (FR-010).
