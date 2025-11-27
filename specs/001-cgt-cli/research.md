# Research: Capital Gains Tax (CGT) CLI Tool

**Feature**: `001-cgt-cli`
**Date**: 2025-11-27

## Decisions & Rationale

### 1. PEG Parser Library: `pest` vs `nom` vs `chumsky`

- **Decision**: **`pest`**
- **Rationale**:
  - User explicitly requested a PEG parser.
  - `pest` uses a separate `.pest` grammar file, which makes the DSL definition declarative and readable ("Deep Modules").
  - Excellent error reporting (Constitution Principle IV: UX).
  - WASM compatible.
- **Alternatives Considered**:
  - `nom`: Faster, but is a parser combinator library, not strict PEG. Code can get verbose.
  - `chumsky`: Great error recovery, but combinator-based.
  - `lalrpop`: LR(1) parser, not PEG.

### 2. Financial Mathematics: `rust_decimal`

- **Decision**: **`rust_decimal`**
- **Rationale**:
  - **Constitution Principle II (Safety)**: Floating point math (f64) is non-negotiable for financial calculations due to rounding errors.
  - `rust_decimal` provides 128-bit fixed-point precision, suitable for currency and tax calculations.
  - Supports serialization via `serde`.
- **Alternatives Considered**:
  - `f64`: Rejected due to precision issues (0.1 + 0.2 != 0.3).
  - `bigdecimal`: Arbitrary precision, but slower and likely overkill for standard CGT. `rust_decimal` fits in stack/registers better.

### 3. Date/Time Library: `chrono`

- **Decision**: **`chrono`**
- **Rationale**:
  - The standard for Rust time.
  - `NaiveDate` is perfect for transaction dates (no time/timezone needed for daily matching rules).
  - `serde` support features.
  - WASM compatible (via `wasm-bindgen` if needed later, or just pure logic).

### 4. CLI Framework: `clap`

- **Decision**: **`clap` (Derive API)**
- **Rationale**:
  - Industry standard for Rust CLIs.
  - "Derive" pattern keeps code declarative and simple (Constitution Principle I).
  - Generates help messages automatically (Constitution Principle IV).

### 5. Workspace Structure

- **Decision**: **Cargo Workspace with `cgt-core` and `cgt-cli`**
- **Rationale**:
  - **WASM Requirement**: `cgt-core` must not depend on system I/O or CLI crates to be easily compiled to WASM later.
  - **Testing**: Allows unit testing core logic independently of CLI argument parsing.

### 6. JSON Serialization

- **Decision**: **`serde` + `serde_json`**
- **Rationale**:
  - Rust ecosystem standard.
  - Required for the "JSON output" and "Schema" requirements.
  - `schemars` will be used to generate the JSON Schema automatically from structs (Constitution Principle II: Automation).

## Unknowns Resolved

- **WASM Friendliness**: `pest`, `rust_decimal`, `chrono`, and `serde` all support WASM targets.
- **PEG**: `pest` fulfills this specific requirement perfectly.
