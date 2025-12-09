# Implementation Plan: Codebase Quality Refactoring

**Branch**: `009-codebase-refactoring` | **Date**: 2025-12-09 | **Spec**: [spec.md](./spec.md)
**Input**: Feature specification from `/specs/009-codebase-refactoring/spec.md`

## Summary

Comprehensive refactoring to eliminate code duplication, improve error handling, externalize configuration, restructure the calculator into isolated matching passes, and adopt template-based formatting. The goal is improved maintainability, consistency, and robustness without changing calculation correctness.

## Technical Context

**Language/Version**: Rust 2024 edition
**Primary Dependencies**: pest (parsing), rust_decimal, chrono, thiserror, anyhow, typst-as-lib, toml (new), tera or minijinja (new for plain text templates)
**Storage**: File-based (TOML for exemption overrides, embedded data for defaults)
**Testing**: cargo test (strict clippy: deny unwrap, expect, panic)
**Target Platform**: CLI (macOS, Linux, Windows)
**Project Type**: Workspace with 4 crates (cgt-core, cgt-cli, cgt-formatter-plain, cgt-formatter-pdf)
**Performance Goals**: Handle files with thousands of transactions efficiently (O(n log n) preferred over O(n²))
**Constraints**: No `.unwrap()` or `.expect()` in production code, maintain backward compatibility
**Scale/Scope**: Single-user CLI tool, tax reports for individual investors

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

| Principle                         | Status  | Notes                                                                                                               |
| --------------------------------- | ------- | ------------------------------------------------------------------------------------------------------------------- |
| I. Deep Modules & Simplicity      | ✅ PASS | Matcher extraction creates deep modules with simple interfaces; formatting consolidation reduces surface complexity |
| II. Safety & Robustness           | ✅ PASS | Explicit error handling for overflow/division-by-zero; validation pass prevents undefined states                    |
| III. Modern Testing Standards     | ✅ PASS | All existing tests preserved; new tests for validation, error messages, and Matcher passes                          |
| IV. User Experience Consistency   | ✅ PASS | Unified formatting policy; actionable parser errors with suggestions                                                |
| V. Performance & Efficiency       | ✅ PASS | Acquisition ledger replaces O(n²) loops; reduced cloning in CLI                                                     |
| VI. Domain Mastery & Verification | ✅ PASS | UK CGT rules well-understood; exemption data from HMRC sources                                                      |

**Gate Result**: PASS - No violations requiring justification.

## Project Structure

### Documentation (this feature)

```text
specs/009-codebase-refactoring/
├── plan.md              # This file
├── research.md          # Phase 0 output
├── data-model.md        # Phase 1 output
├── quickstart.md        # Phase 1 output
├── contracts/           # Phase 1 output (internal module interfaces)
└── tasks.md             # Phase 2 output (/speckit.tasks command)
```

### Source Code (repository root)

```text
crates/
├── cgt-core/
│   ├── src/
│   │   ├── lib.rs
│   │   ├── models.rs
│   │   ├── error.rs
│   │   ├── parser.rs           # Enhanced with better error messages
│   │   ├── exemption.rs        # Refactored to load from embedded TOML + optional override
│   │   ├── calculator.rs       # Refactored into Matcher + passes
│   │   ├── matcher/            # NEW: Isolated matching logic
│   │   │   ├── mod.rs
│   │   │   ├── same_day.rs
│   │   │   ├── bed_and_breakfast.rs
│   │   │   ├── section104.rs
│   │   │   └── acquisition_ledger.rs
│   │   ├── validation.rs       # NEW: Input validation pass
│   │   └── formatting.rs       # NEW: Shared formatting utilities
│   ├── data/
│   │   └── exemptions.toml     # NEW: Embedded exemption data
│   └── tests/
├── cgt-formatter-plain/
│   ├── src/
│   │   ├── lib.rs              # Refactored to use templates
│   │   └── templates/          # NEW: Plain text templates
│   │       └── report.txt.tera
│   └── tests/
├── cgt-formatter-pdf/
│   ├── src/
│   │   ├── lib.rs              # Refactored to use shared formatting
│   │   └── templates/
│   │       └── report.typ
│   └── tests/
└── cgt-cli/
    ├── src/
    │   ├── main.rs             # Reduced cloning, improved data flow
    │   └── commands.rs
    └── tests/
```

**Structure Decision**: Existing workspace structure preserved. New `matcher/` module extracted from calculator.rs. New `formatting.rs` in cgt-core for shared utilities. New `data/` directory for embedded exemption TOML.
