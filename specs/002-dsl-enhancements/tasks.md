______________________________________________________________________

## description: "Task list for 002-dsl-enhancements"

# Tasks: DSL Enhancements

**Input**: Design documents from `/specs/002-dsl-enhancements/`
**Prerequisites**: plan.md, spec.md, data-model.md, contracts/, research.md

**Tests**: Integration tests using `assert_cmd` and TDD approach for core logic.

**Organization**: Tasks grouped by user story.

## Phase 1: Setup (Shared Infrastructure)

**Purpose**: Project initialization and basic structure

- [x] T001 Create cargo workspace structure (already exists)
- [x] T002 Initialize `crates/cgt-core` (already exists)
- [x] T003 Initialize `crates/cgt-cli` (already exists)
- [x] T004 [P] Configure clippy and formatting rules (already exists)
- [x] T005 [P] Set up `tests/` directory (already exists)

______________________________________________________________________

## Phase 2: Foundational (Blocking Prerequisites)

**Purpose**: Core data structures and common utilities

- [x] T006 Create `crates/cgt-core/src/models.rs` (already exists)
- [x] T007 Implement `serde` serialization/deserialization (already exists)
- [x] T008 Create `crates/cgt-core/src/lib.rs` (already exists)
- [x] T009 Define `Section104Holding` struct (already exists)
- [x] T010 Define `Match` and `TaxReport` structs (already exists)
- [x] T011 Create error types (already exists)

**Checkpoint**: Core models exist and compile.

______________________________________________________________________

## Phase 3: User Story 1 - Enhanced DSL Readability (Priority: P1)

**Goal**: Implement the new DSL syntax for improved readability.

**Independent Test**: Provide sample transaction files with the new DSL syntax. Verify successful parsing and generation of the internal representation without errors.

### Implementation for User Story 1

- [ ] T012 Update `crates/cgt-core/src/parser.pest` for `DIVIDEND` with `TAX` keyword.
- [ ] T013 Update `crates/cgt-core/src/models.rs` `Operation::Dividend` to include `tax_keyword: String`.
- [ ] T014 Update `crates/cgt-core/src/parser.rs` to parse `DIVIDEND` with `TAX` keyword.
- [ ] T015 Update `crates/cgt-core/src/parser.pest` for `CAPRETURN` with `EXPENSES` keyword.
- [ ] T016 Update `crates/cgt-core/src/models.rs` `Operation::CapReturn` to include `expenses_keyword: String`.
- [ ] T017 Update `crates/cgt-core/src/parser.rs` to parse `CAPRETURN` with `EXPENSES` keyword.
- [ ] T018 Update `crates/cgt-core/src/parser.pest` for `SPLIT/UNSPLIT` with `RATIO` keyword.
- [ ] T019 Update `crates/cgt-core/src/models.rs` `Operation::Split` and `Operation::Unsplit` to include `split_unsplit_keyword: String`.
- [ ] T020 Update `crates/cgt-core/src/parser.rs` to parse `SPLIT/UNSPLIT` with `RATIO` keyword.
- [ ] T021 Update `crates/cgt-core/src/parser.pest` for flexible whitespace and comments.
- [ ] T022 Update all `tests/data/*.cgt` files to the new DSL syntax.
- [ ] T023 Update `specs/002-dsl-enhancements/quickstart.md` with new DSL examples.
- [ ] T024 Update `README.md` with new DSL examples.

**Checkpoint**: CLI can parse new DSL syntax and produce expected internal representation.

______________________________________________________________________

## Phase 4: User Story 2 - Robust Test Suite & Output Validation (Priority: P1)

**Goal**: Ensure test suite reliability and validate outputs against original `cgtcalc` data.

**Independent Test**: Re-run the entire test suite after applying sorting and format changes. Verify that all assertions pass and that generated reports match original `cgtcalc` outputs after re-conversion.

### Implementation for User Story 2

- [ ] T025 Ensure all internal transaction file references consistently use the `.cgt` extension.
- [ ] T026 Sort transactions within all `tests/data/*.cgt` files chronologically.
- [ ] T027 Download original `cgtcalc` test data (inputs/outputs) to a temporary location.
- [ ] T028 Re-convert original `cgtcalc` input data to new `.cgt` format, ensuring chronological order.
- [ ] T029 Re-convert original `cgtcalc` output data to `.json` format, using precise calculations.
- [ ] T030 Run `cargo test` to verify every single test case and make sure that every output in tests is correct.
- [ ] T031 Add acceptance test for new DSL (`DIVIDEND TAX`, `CAPRETURN EXPENSES`, `SPLIT RATIO`).
- [ ] T032 Refactor `crates/cgt-core/tests/matching_tests.rs` to display every single test file as a separate test case.

**Checkpoint**: Test suite passes, and outputs are verified against original `cgtcalc` data.

______________________________________________________________________

## Phase 5: Polish & Cross-Cutting Concerns

**Purpose**: Documentation, performance, and final checks.

- [ ] T033 Review code for any remaining hardcoded `.txt` extensions and update to `.cgt`.
- [ ] T034 Run `cargo clippy --fix` and `cargo fmt` to apply fixes and format.

______________________________________________________________________

## Dependencies & Execution Order

### Phase Dependencies

- **Setup (Phase 1)**: Blocks everything.
- **Foundational (Phase 2)**: Blocks all User Stories.
- **User Story 1 (Phase 3)**: Blocks User Story 2 (need parsing before validating).
- **User Story 2 (Phase 4)**: Depends on US1 completion.

### User Story Dependencies

- **US1**: Independent after Foundational.
- **US2**: Depends on US1 (parser changes) and Foundational (models).

## Implementation Strategy

### MVP First

1. Complete Phases 1 & 2.
2. Complete Phase 3 (Parser changes and DSL updates).
3. Complete Phase 4 (Test suite validation).

### Parallel Opportunities

- Parsing logic updates for different commands (T012, T014, T016) can be done in parallel.
- Documentation updates (T020, T021) can be done in parallel once the DSL is stable.
- Test data conversion (T024, T025, T026) can be done in parallel.
