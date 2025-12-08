# Implementation Plan: Internal Data Model Improvements

**Branch**: `005-internal-data-model` | **Date**: 2025-12-08 | **Spec**: [spec.md](./spec.md)
**Input**: Feature specification from `/specs/005-internal-data-model/spec.md`

## Summary

Refactor the internal data model (`TaxReport`, `Match`) to better represent UK CGT domain concepts. Key changes:

- Introduce `TaxPeriod` type for validated UK tax year notation ("2023/24")
- Restructure flat `matches` array into hierarchical `disposals[].matches[]`
- Add `TaxYearSummary` to group disposals by tax year with totals
- Include `acquisition_date` on B&B matches for audit trail
- Clean up decimal precision in JSON output (2 decimal places)

## Technical Context

**Language/Version**: Rust 2024 edition
**Primary Dependencies**: serde, serde_json, chrono, rust_decimal, thiserror, schemars (JsonSchema)
**Storage**: N/A (in-memory data structures, JSON serialization)
**Testing**: cargo test (existing test infrastructure)
**Target Platform**: CLI tool (cross-platform)
**Project Type**: Single Rust workspace with two crates (cgt-core, cgt-cli)
**Performance Goals**: N/A (data model refactoring, no performance-critical changes)
**Constraints**: Must maintain JSON serialization compatibility for test validation
**Scale/Scope**: ~80 lines of models.rs, ~24 test files to migrate

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

| Principle                         | Status  | Notes                                                              |
| --------------------------------- | ------- | ------------------------------------------------------------------ |
| I. Deep Modules & Simplicity      | ✅ Pass | TaxPeriod encapsulates validation, Disposal groups related matches |
| II. Safety & Robustness           | ✅ Pass | TaxPeriod prevents invalid states at compile/parse time            |
| III. Modern Testing Standards     | ✅ Pass | All existing tests preserved; new model requires test migration    |
| IV. User Experience Consistency   | ✅ Pass | Cleaner JSON output with "2023/24" format                          |
| V. Performance & Efficiency       | ✅ Pass | No performance impact                                              |
| VI. Domain Mastery & Verification | ✅ Pass | Model aligns with HMRC CGT concepts                                |

**All gates pass. Proceeding to Phase 0.**

## Project Structure

### Documentation (this feature)

```text
specs/005-internal-data-model/
├── plan.md              # This file
├── research.md          # Phase 0 output
├── data-model.md        # Phase 1 output
├── quickstart.md        # Phase 1 output
└── tasks.md             # Phase 2 output (via /speckit.tasks)
```

### Source Code (repository root)

```text
crates/
├── cgt-core/
│   └── src/
│       ├── lib.rs
│       ├── models.rs      # PRIMARY: Data model changes here
│       ├── calculator.rs  # Update to produce new model
│       ├── parser.rs
│       └── error.rs
├── cgt-cli/
│   └── src/
│       └── main.rs        # Minor: CLI output formatting
tests/
└── data/
    ├── *.cgt              # Input files (unchanged)
    └── *.json             # Expected output files (migrate to new format)
```

**Structure Decision**: Existing crate structure is appropriate. Changes are localized to `cgt-core/src/models.rs` with cascading updates to `calculator.rs` and test expected files.
