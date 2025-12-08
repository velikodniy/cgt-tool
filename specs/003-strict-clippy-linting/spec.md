# Feature Specification: Strict Clippy Linting

**Feature Branch**: `003-strict-clippy-linting`
**Created**: 2025-12-08
**Status**: Draft
**Input**: User description: "Remove all allows for Clippy. The linters should be strict. Clippy config should not allow .unwrap(). All unwraps should be removed with proper error handling. Users should see meaningful error messages."

## Clarifications

### Session 2025-12-08

- Q: Which error handling library to use? → A: Keep `thiserror` (current) - stable, zero-cost, idiomatic
- Q: How to configure Clippy? → A: Use `clippy.toml` at workspace root - centralized, modern approach
- Q: Additional strict lints beyond `unwrap_used`? → A: Enable curated strict lints (`expect_used` in production code, `panic`, `todo`, `unimplemented`)

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Developer Builds Project with Strict Linting (Priority: P1)

A developer clones the repository and runs `cargo clippy`. The linter enforces strict rules with no suppressed warnings, ensuring code quality is maintained across the project.

**Why this priority**: Strict linting is the foundation of code quality. Without enforced linting rules, other improvements cannot be consistently maintained.

**Independent Test**: Can be fully tested by running `cargo clippy` and verifying zero warnings are suppressed and all checks pass.

**Acceptance Scenarios**:

1. **Given** a fresh clone of the repository, **When** running `cargo clippy`, **Then** the build completes with no warnings and no `#[allow(...)]` attributes in the codebase
2. **Given** a developer adds code with `.unwrap()`, **When** running `cargo clippy`, **Then** the linter fails with a clear error indicating unwrap is not permitted
3. **Given** a developer adds `#[allow(clippy::...)]` attribute, **When** running `cargo clippy`, **Then** the linter fails indicating allow attributes are forbidden

---

### User Story 2 - User Receives Meaningful Error for Malformed Input (Priority: P1)

A user runs the CLI tool with a malformed input file. Instead of a panic or cryptic error, the user sees a clear, actionable error message explaining what went wrong and where.

**Why this priority**: User experience is critical. Panics from unwrap() provide no useful information and make the tool appear broken.

**Independent Test**: Can be fully tested by providing malformed input files and verifying error messages are descriptive and actionable.

**Acceptance Scenarios**:

1. **Given** an input file with invalid date format, **When** parsing the file, **Then** the error message indicates the specific line number and explains the expected date format
2. **Given** an input file with missing required fields, **When** parsing the file, **Then** the error message specifies which field is missing and on which line
3. **Given** an input file with invalid numeric values, **When** parsing the file, **Then** the error message shows the invalid value and expected format

---

### User Story 3 - Developer Gets Helpful Errors During Development (Priority: P2)

A developer working on the codebase encounters an internal error condition. The error propagates with context rather than panicking, making debugging straightforward.

**Why this priority**: Good error handling in internal code reduces debugging time and makes the codebase more maintainable.

**Independent Test**: Can be fully tested by triggering edge cases in calculator and parser modules and verifying errors contain sufficient context.

**Acceptance Scenarios**:

1. **Given** an edge case in tax year calculation, **When** the calculation encounters an unexpected condition, **Then** the error includes context about what operation failed
2. **Given** parser encounters unexpected token structure, **When** parsing proceeds, **Then** the error explains what was expected vs what was found

---

### Edge Cases

- What happens when date creation fails (e.g., invalid year like year 0 or far future dates)?
- How does the system handle parsing errors deep in nested grammar rules?
- What happens when multiple errors occur in sequence during parsing?

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: The project MUST have no `#[allow(...)]` attributes in any source files (excluding generated code in target/)
- **FR-002**: The project MUST configure Clippy via `clippy.toml` at workspace root to deny:
  - `clippy::unwrap_used` - forbid `.unwrap()` calls
  - `clippy::expect_used` - forbid `.expect()` in production code (allowed in tests)
  - `clippy::panic` - forbid `panic!()` macro in production code
  - `clippy::todo` - forbid `todo!()` macro
  - `clippy::unimplemented` - forbid `unimplemented!()` macro
- **FR-003**: All current `.unwrap()` calls in production code MUST be replaced with proper error handling that returns `Result` types
- **FR-004**: All `.unwrap()` calls in test code MUST be replaced with `.expect("descriptive message")` or proper error handling
- **FR-005**: Error messages from parsing failures MUST include the line number where the error occurred
- **FR-006**: Error messages from parsing failures MUST describe what was expected vs what was found
- **FR-007**: The project MUST pass `cargo clippy` with zero warnings after all changes
- **FR-008**: Date creation failures MUST return descriptive errors rather than panicking

### Key Entities

- **CgtError**: The existing error type that must be extended to handle all error cases currently using unwrap
- **Parser functions**: Functions in parser.rs that currently use unwrap to extract parsed tokens
- **Calculator functions**: Functions in calculator.rs that use unwrap for date creation

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: Running `cargo clippy` produces zero warnings and zero suppressed lints
- **SC-002**: The codebase contains zero `#[allow(...)]` attributes in source files
- **SC-003**: The codebase contains zero `.unwrap()` calls in production code (src/ directories)
- **SC-004**: All error messages from malformed input include line numbers and descriptive context
- **SC-005**: 100% of existing tests continue to pass after refactoring
- **SC-006**: No panics occur when processing malformed input files (graceful error handling)

## Assumptions

- Test files may use `.expect()` with descriptive messages (tests are exempt from `expect_used` lint)
- The existing `CgtError` type (using `thiserror`) will be extended to cover new error cases
- Clippy configuration will be centralized in `clippy.toml` at workspace root
