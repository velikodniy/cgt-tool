# Implementation Plan: Capital Gains Tax (CGT) CLI Tool

**Branch**: `001-cgt-cli` | **Date**: 2025-11-27 | **Spec**: [specs/001-cgt-cli/spec.md](spec.md)
**Input**: Feature specification from `/specs/001-cgt-cli/spec.md`

## Summary

Build a high-performance, modular CLI tool in Rust to calculate Capital Gains Tax (CGT) for UK assets. The system features a core library (`cgt-core`) with PEG-based parsing for a human-readable DSL (including optional expenses and comments) and exact financial calculations, and a CLI binary (`cgt-cli`) for user interaction and reporting. The architecture is designed to be WASM-friendly for future web integration.

## Technical Context

**Language/Version**: Rust (Stable)
**Primary Dependencies**:

- `pest` (PEG parser)
- `rust_decimal` (Financial math)
- `serde`/`serde_json` (Serialization)
- `clap` (CLI)
- `chrono` (Date/Time)
  **Storage**: N/A (CLI operates on input files)
  **Testing**: `cargo test` (Unit/Integration), `assert_cmd` (CLI tests)
  **Target Platform**: CLI (Linux/macOS/Windows), WASM-compatible core
  **Project Type**: Workspace (Lib + Bin)
  **Performance Goals**: Parse & process 1000 transactions < 1s
  **Constraints**: Strict FIFO, UK CGT rules (Same Day, B&B, S104), Zero data loss (Decimal)
  **Scale/Scope**: Personal finance scale (hundreds/thousands of transactions)

### DSL Specifics

The Domain Specific Language (DSL) for transactions now supports:

- Flexible whitespace: Multiple spaces between elements are allowed and ignored.
- Comments: Lines starting with `#` are treated as comments and ignored by the parser.
- Improved readability for `BUY`/`SELL` operations: `AMOUNT @ PRICE [EXPENSES EXPENSE_AMOUNT]`, where `EXPENSES` is an optional keyword followed by its value.

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

- **I. Deep Modules & Simplicity**: ✅ Splitting `core` (logic) and `cli` (interface) ensures deep modules. `core` handles complex matching rules behind a simple `calculate()` API.
- **II. Safety & Robustness**: ✅ Rust ownership model + `rust_decimal` prevents memory errors and floating-point inaccuracies. Strong typing for transaction variants.
- **III. Modern Testing Standards**: ✅ Plan includes TDD with `cargo test` and integration tests using `assert_cmd` to verify CLI behavior against spec scenarios.
- **IV. User Experience Consistency**: ✅ `clap` provides standard CLI help/errors. `pest` allows custom error reporting for DSL syntax issues.
- **V. Performance & Efficiency**: ✅ Rust + PEG parser meets the \<1s performance goal easily. Zero-copy parsing where possible (though strict separation might require ownership).

## Project Structure

### Documentation (this feature)

```text
specs/001-cgt-cli/
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
├── cgt-core/           # Domain logic, Parser, Calculator (WASM-friendly)
│   ├── src/
│   │   ├── lib.rs
│   │   ├── models.rs
│   │   ├── parser.rs   # PEG grammar & logic
│   │   └── calculator.rs # CGT Rules (Same Day, B&B, S104)
│   └── Cargo.toml
└── cgt-cli/            # CLI Binary
    ├── src/
    │   ├── main.rs
    │   └── commands.rs
    └── Cargo.toml

tests/                  # Integration tests
├── cli_tests.rs        # assert_cmd tests
└── matching_tests.rs   # Complex scenario tests
```

**Structure Decision**: Workspace pattern chosen to enforce separation of concerns (Core vs CLI) and ensure `cgt-core` remains pure and WASM-compilable without CLI dependencies.

## Complexity Tracking

| Violation            | Why Needed           | Simpler Alternative Rejected Because     |
| -------------------- | -------------------- | ---------------------------------------- |
| Workspace (2 crates) | WASM future-proofing | Monolith prevents easy WASM export later |
