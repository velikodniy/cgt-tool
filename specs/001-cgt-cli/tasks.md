______________________________________________________________________

## description: "Task list for 001-cgt-cli"

# Tasks: Capital Gains Tax (CGT) CLI Tool

**Input**: Design documents from `/specs/001-cgt-cli/`
**Prerequisites**: plan.md, spec.md, data-model.md, contracts/, research.md

**Tests**: Integration tests using `assert_cmd` and TDD approach for core logic (requested in plan/constitution).

**Organization**: Tasks grouped by user story.

## Phase 1: Setup (Shared Infrastructure)

**Purpose**: Project initialization and basic structure

- [x] T001 Create cargo workspace structure with `crates/cgt-core` and `crates/cgt-cli`
- [x] T002 Initialize `crates/cgt-core` as a library with `pest`, `rust_decimal`, `serde`, `chrono` dependencies
- [x] T003 Initialize `crates/cgt-cli` as a binary with `clap`, `anyhow`, `serde_json` dependencies
- [x] T004 [P] Configure clippy and formatting rules in `.cargo/config.toml` or `rustfmt.toml`
- [x] T005 [P] Set up `tests/` directory for workspace-level integration tests

______________________________________________________________________

## Phase 2: Foundational (Blocking Prerequisites)

**Purpose**: Core data structures and common utilities

- [x] T006 Create `crates/cgt-core/src/models.rs` with `Transaction`, `Action`, and `Operation` enums (per data-model.md)
- [x] T007 Implement `serde` serialization/deserialization for all models in `cgt-core`
- [x] T008 Create `crates/cgt-core/src/lib.rs` and export public modules
- [x] T009 Define `Section104Holding` struct in `crates/cgt-core/src/models.rs`
- [x] T010 Define `Match` and `TaxReport` structs in `crates/cgt-core/src/models.rs`
- [x] T011 Create error types in `crates/cgt-core/src/error.rs` (using `thiserror` if helpful, or std::error)

**Checkpoint**: Core models exist and compile.

______________________________________________________________________

## Phase 3: User Story 1 - DSL Parsing & Validation (Priority: P1)

**Goal**: Convert raw text DSL into structured JSON.

**Independent Test**: `cgt-cli parse transactions.txt` outputs valid JSON.

### Tests for User Story 1

- [x] T012 [US1] Create `crates/cgt-core/tests/parser_tests.rs` with failing tests for valid/invalid DSL lines
- [x] T013 [US1] Create `tests/cli_tests.rs` with `assert_cmd` test for `parse` command

### Implementation for User Story 1

- [x] T014 [US1] Create `crates/cgt-core/src/parser.pest` grammar file definition
- [x] T015 [US1] Implement `crates/cgt-core/src/parser.rs` using `pest` derive
- [x] T016 [US1] Implement parsing logic for `BUY` and `SELL` actions
- [x] T017 [US1] Implement parsing logic for `DIVIDEND`, `CAPRETURN` actions
- [x] T018 [US1] Implement parsing logic for `SPLIT`, `UNSPLIT` actions
- [x] T019 [US1] Implement `cgt-cli` "parse" subcommand in `crates/cgt-cli/src/commands.rs`
- [x] T020 [US1] Implement JSON output formatting for parse command
- [x] T021 [US1] Add `--schema` flag to parse command to dump JSON schema (using `schemars`)

**Checkpoint**: CLI can parse DSL file and output JSON or error.

______________________________________________________________________

## Phase 4: User Story 2 - Capital Gains Report Generation (Priority: P1)

**Goal**: Calculate tax liability using UK CGT rules.

**Independent Test**: `cgt-cli report transactions.txt` outputs correct gain/loss report.

### Tests for User Story 2

- [x] T022 [US2] Create `crates/cgt-core/tests/matching_tests.rs` and use data-driven tests from `tests/data/` (`.cgt` and `.json` pairs)
- [x] T023 [US2] Add `assert_cmd` test for `report` command in `tests/cli_tests.rs`

### Implementation for User Story 2

- [x] T024 [US2] Create `crates/cgt-core/src/calculator.rs` module
- [x] T025 [US2] Implement "Same Day" matching rule logic
- [x] T026 [US2] Implement "Bed and Breakfast" (30-day) matching rule logic
- [x] T027 [US2] Implement "Section 104" pool logic (average cost handling)
- [x] T028 [US2] Implement `calculate()` function to orchestrate processing (sort, process, aggregate)
- [x] T029 [US2] Implement `cgt-cli` "report" subcommand in `crates/cgt-cli/src/commands.rs`
- [x] T030 [US2] Format output using `TaxReport` struct and serde_json

**Checkpoint**: CLI can generate accurate tax reports.

______________________________________________________________________

## Phase 5: Polish & Cross-Cutting Concerns

**Purpose**: Documentation, performance, and final checks.

- [x] T031 Ensure WASM compatibility check (run `cargo check --target wasm32-unknown-unknown -p cgt-core`)
- [x] T032 Update README.md with usage examples
- [x] T033 Run clippy and fix all warnings
- [x] T034 Verify error messages are user-friendly (no raw stack traces)

______________________________________________________________________

## Phase 6: Post-Implementation Verification

**Purpose**: Deep-dive validation of test data and results.

- [ ] T035 Verify all `tests/data/*.json` outputs against the original `cgtcalc` text reports for precision and rounding discrepancies. Download original `cgtcalc` test data (inputs/outputs) if local copies are not sufficient to compare calculations. We should trust the original tests.
- [ ] T036 Ensure all `tests/data/*.cgt` input files are sorted chronologically by date.

______________________________________________________________________

## Dependencies & Execution Order

### Phase Dependencies

- **Setup (Phase 1)**: Blocks everything.
- **Foundational (Phase 2)**: Blocks all User Stories.
- **User Story 1**: Blocks User Story 2 (need parsing before calculating).
- **User Story 2**: Depends on US1.
- **Post-Implementation Verification (Phase 6)**: Depends on all prior phases.

### User Story Dependencies

- **US1**: Independent after Foundational.
- **US2**: Depends on US1 (parser) and Foundational (models).

## Implementation Strategy

### MVP First

1. Complete Phases 1 & 2 (Models).
2. Complete Phase 3 (Parsing) -> Deliverable: "Parser Tool".
3. Complete Phase 4 (Calculation) -> Deliverable: "Tax Tool".

### Parallel Opportunities

- **Parsing Logic**: Individual action parsers (T016, T017, T018) can be written in parallel.
- **Matching Rules**: Same Day (T025), B&B (T026), and S104 (T027) logic can be implemented in parallel once the calculator skeleton exists.
