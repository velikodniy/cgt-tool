# Implementation Plan: DSL Enhancements

**Branch**: `002-dsl-enhancements` | **Date**: 2025-11-27 | **Spec**: [specs/002-dsl-enhancements/spec.md](spec.md)
**Input**: Feature specification from `/specs/002-dsl-enhancements/spec.md`

## Summary

This plan outlines the implementation for enhancing the Domain Specific Language (DSL) used by the CGT CLI tool. This includes incorporating more readable keywords (`TAX`, `EXPENSES`, `SPLIT` keyword for SPLIT/UNSPLIT commands), ensuring consistent use of the `.cgt` file extension for transaction files, and establishing a robust process for test suite validation against trusted original `cgtcalc` outputs to guarantee calculation accuracy.

## Technical Context

**Language/Version**: Rust (Stable)
**Primary Dependencies**:

- `pest` (PEG parser - will require grammar updates)
- `rust_decimal` (Financial math - unaffected)
- `serde`/`serde_json` (Serialization - unaffected)
- `clap` (CLI - unaffected)
- `chrono` (Date/Time - unaffected)
  **Storage**: N/A (CLI operates on input files)
  **Testing**: `cargo test` (Unit/Integration), `assert_cmd` (CLI tests - will be heavily utilized for validation)
  **Target Platform**: CLI (Linux/macOS/Windows)
  **Project Type**: Workspace (Lib + Bin)
  **Performance Goals**: Parse & process 1000 transactions < 1s (unaffected by DSL changes)
  **Constraints**: Strict FIFO, UK CGT rules (Same Day, B&B, S104), Zero data loss (Decimal)
  **Scale/Scope**: Personal finance scale (hundreds/thousands of transactions)

### DSL Specifics

The Domain Specific Language (DSL) for transactions will be enhanced to support:

- **More Readable Commands**: Keywords like `TAX`, `EXPENSES`, and `SPLIT` will be embedded within commands.
  - `DIVIDEND` format: `YYYY-MM-DD DIVIDEND TICKER AMOUNT TAX TAX_AMOUNT`
  - `CAPRETURN` format: `YYYY-MM-DD CAPRETURN TICKER AMOUNT EXPENSES EXPENSE_AMOUNT`
  - `SPLIT/UNSPLIT` format: `YYYY-MM-DD SPLIT FOO RATIO RATIO_VALUE` or `YYYY-MM-DD UNSPLIT FOO RATIO RATIO_VALUE`
- **Flexible Whitespace**: Multiple spaces between elements are allowed and ignored by the parser.
- **Comments**: Lines starting with `#` are treated as comments and ignored by the parser.

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

- **I. Deep Modules & Simplicity**: ✅ Grammar changes keep parsing logic encapsulated in `parser.pest` and `parser.rs`. The calculator logic remains separate and unaffected by DSL syntax.
- **II. Safety & Robustness**: ✅ Parsing changes will be rigorously tested to ensure no new errors are introduced. `rust_decimal` ensures continued precision. Test suite re-validation enhances overall reliability.
- **III. Modern Testing Standards**: ✅ Feature directly mandates rigorous test suite re-validation, including sorting inputs and trusting original `cgtcalc` outputs, reinforcing TDD and quality.
- **IV. User Experience Consistency**: ✅ Improves DSL readability directly, enhancing CLI usability and reducing user error.
- **V. Performance & Efficiency**: ✅ Grammar enhancements are not expected to negatively impact parsing performance. `pest` remains efficient.

## Project Structure

### Documentation (this feature)

```text
specs/002-dsl-enhancements/
├── plan.md              # This file
├── research.md          # Phase 0 output
├── data-model.md        # Phase 1 output
├── quickstart.md        # Phase 1 output
├── contracts/           # Phase 1 output
└── tasks.md             # Phase 2 output
```

### Source Code (repository root)

```text
Cargo.toml (Workspace)
crates/
├── cgt-core/           # Domain logic, Parser, Calculator
│   ├── src/
│   │   ├── lib.rs
│   │   ├── models.rs
│   │   ├── parser.rs   # Will be updated for new grammar
│   │   └── calculator.rs
│   └── Cargo.toml
└── cgt-cli/            # CLI Binary
    ├── src/
    │   ├── main.rs
    │   └── commands.rs
    └── Cargo.toml

tests/                  # Integration tests
├── cli_tests.rs
└── matching_tests.rs   # Will be updated for re-validation and sorting
```

**Structure Decision**: Existing workspace pattern is maintained. Updates are localized within `cgt-core` (parser) and `tests/data`.

## Complexity Tracking

| Violation | Why Needed                                    | Simpler Alternative Rejected Because |
| --------- | --------------------------------------------- | ------------------------------------ |
| N/A       | (No new significant architectural complexity) | (N/A)                                |
