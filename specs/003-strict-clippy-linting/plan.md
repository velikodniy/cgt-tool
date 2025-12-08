# Implementation Plan: Strict Clippy Linting

**Branch**: `003-strict-clippy-linting` | **Date**: 2025-12-08 | **Spec**: [spec.md](spec.md)
**Input**: Feature specification from `/specs/003-strict-clippy-linting/spec.md`

## Summary

Enforce strict Clippy linting by creating a centralized `clippy.toml` configuration that denies `unwrap_used`, `expect_used`, `panic`, `todo`, and `unimplemented` in production code. Remove all existing `#[allow(...)]` attributes and replace all `.unwrap()` calls with proper error handling using `thiserror`. Users will receive meaningful, contextual error messages instead of panics.

## Technical Context

**Language/Version**: Rust 2024 edition
**Primary Dependencies**: pest (parsing), thiserror (error types), anyhow (CLI error handling), chrono (dates), rust_decimal (numbers)
**Storage**: N/A (file-based input processing)
**Testing**: cargo test, assert_cmd for CLI integration tests
**Target Platform**: CLI tool (cross-platform)
**Project Type**: Workspace with two crates (cgt-core library, cgt-cli binary)
**Performance Goals**: N/A (code quality change, no performance impact expected)
**Constraints**: Backward compatible - existing valid inputs must continue to work
**Scale/Scope**: Small codebase - 2 allow attributes, ~20 unwrap calls in production code

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

| Principle                         | Status  | Notes                                                                                 |
| --------------------------------- | ------- | ------------------------------------------------------------------------------------- |
| I. Deep Modules & Simplicity      | ✅ PASS | Error handling improves interface clarity - errors become explicit rather than panics |
| II. Safety & Robustness           | ✅ PASS | Directly implements "Error handling must be explicit, graceful, and actionable"       |
| III. Modern Testing Standards     | ✅ PASS | Existing tests preserved; test code uses `.expect()` with descriptive messages        |
| IV. User Experience Consistency   | ✅ PASS | Users receive "clear feedback, actionable error messages" instead of panics           |
| V. Performance & Efficiency       | ✅ PASS | No performance impact - Result propagation has negligible overhead                    |
| VI. Domain Mastery & Verification | ✅ PASS | Error messages include domain context (line numbers, expected formats)                |

**Code Quality Gates**:

- ✅ Automated Tests: Will maintain 100% pass rate
- ✅ Linting/Formatting: This feature enforces stricter linting (zero tolerance)
- ✅ Review: Changes improve readability through explicit error handling

**Commit Discipline**: Each logical change will be committed separately:

1. Add `clippy.toml` with strict lint configuration
2. Remove allow attributes (fix underlying issues)
3. Replace unwraps in parser.rs with proper error handling
4. Replace unwraps in calculator.rs with proper error handling
5. Update test code to use expect() with descriptive messages

## Project Structure

### Documentation (this feature)

```text
specs/003-strict-clippy-linting/
├── plan.md              # This file
├── research.md          # Phase 0 output
├── data-model.md        # Phase 1 output (error type extensions)
├── quickstart.md        # Phase 1 output
└── tasks.md             # Phase 2 output (/speckit.tasks command)
```

### Source Code (repository root)

```text
clippy.toml                  # NEW: Workspace-level Clippy configuration

crates/
├── cgt-core/
│   ├── src/
│   │   ├── lib.rs           # No changes needed (config in clippy.toml)
│   │   ├── error.rs         # Extend CgtError with new variants
│   │   ├── parser.rs        # Replace ~18 unwrap() calls
│   │   ├── calculator.rs    # Replace 2 unwrap() calls, remove allow attribute
│   │   └── models.rs
│   └── tests/
│       ├── parser_tests.rs  # Remove allow attribute, use expect()
│       └── matching_tests.rs # Use expect() for test assertions
└── cgt-cli/
    ├── src/
    │   ├── main.rs          # No changes needed (config in clippy.toml)
    │   └── commands.rs
    └── tests/
        └── cli_tests.rs
```

**Structure Decision**: Existing workspace structure preserved. Clippy configuration centralized in workspace root `clippy.toml`. Changes are internal refactoring within existing files.

## Complexity Tracking

No constitution violations. This change reduces complexity by making error handling explicit and centralizing lint configuration.
