# Implementation Plan: Better Testing Coverage

**Branch**: `010-better-testing` | **Date**: 2025-12-09 | **Spec**: specs/010-better-testing/spec.md
**Input**: Feature specification from `/specs/010-better-testing/spec.md`

## Summary

Add targeted regression tests and reporting assertions for CGT calculations: 2024/25 pre/post 30 Oct rate split, accumulation dividends and CAPRETURN cost adjustments, expenses/rounding correctness, B&B reporting quantity, FX guardrail, and text-report clarity (PDF parity out of scope). No production code changes planned unless tests expose defects.

## Technical Context

**Language/Version**: Rust 2024 edition (workspace uses stable Rust)
**Primary Dependencies**: rust_decimal, chrono, pest (DSL parsing), anyhow/thiserror (errors), typst-as-lib (formatting; PDF not in scope for this feature), cargo workspace crates `cgt-core`, `cgt-cli`, formatters
**Storage**: None (in-memory processing of fixtures)
**Testing**: `cargo test` (unit/integration), fixture-based assertions under `tests/`
**Target Platform**: Cross-platform CLI/library (macOS/Linux/Windows)
**Project Type**: CLI + core library in a single repo/workspace
**Performance Goals**: Test suite completes within a few minutes; individual new fixtures keep runtime negligible
**Constraints**: Do not remove or modify existing tests without proof; no external network/API calls; adhere to HMRC rules for expected outputs
**Scale/Scope**: Test fixtures sized to existing patterns (single-file inputs in `tests/inputs`, matching expected outputs in `tests/json`/`tests/plain`)

## Constitution Check

Gates to honor before/after design:

- Principle III (Modern Testing Standards): Only add tests; never delete/alter existing tests without proof.
- Principle VI (Domain Mastery & Verification): Expected results must follow HMRC rules (Oct 30, 2024 rate change; accumulation dividend and CAPRETURN treatments).
- Safety/Robustness: Tests must assert explicit, deterministic behavior (no silent failures).
  No anticipated violations; no complexity tracking required.

## Project Structure

### Documentation (this feature)

```text
specs/010-better-testing/
├── plan.md              # This plan
├── research.md          # Phase 0 output
├── data-model.md        # Phase 1 output
├── quickstart.md        # Phase 1 output
├── contracts/           # Phase 1 output (N/A API; note recorded)
└── tasks.md             # Phase 2 (not generated here)
```

### Source Code (repository root)

```text
crates/
├── cgt-core/            # Core CGT logic, DSL parser, calculators
│   ├── src/
│   └── tests/
├── cgt-cli/             # CLI wiring/tests
└── cgt-formatter-*      # Formatters (PDF/text); PDF parity out of scope

tests/
├── inputs/              # CGT input fixtures
├── json/                # Expected JSON outputs
└── plain/               # Expected text outputs
```

**Structure Decision**: Use existing Cargo workspace layout; new fixtures/tests live in `tests/` (and crate-specific tests if needed). Documentation remains under `specs/010-better-testing/`.

## Complexity Tracking

None.
