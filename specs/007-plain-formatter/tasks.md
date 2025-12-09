# Tasks: Plain Text Report Formatter

**Input**: Design documents from `/specs/007-plain-formatter/`
**Prerequisites**: plan.md, spec.md, research.md, data-model.md, quickstart.md

**Tests**: Required - spec explicitly requests verification against cgtcalc reference outputs

**Organization**: Tasks grouped by user story. US1 (Plain Text Formatter) is the core MVP; US2 (CLI Format Selection) and US3 (Extensible Architecture) are naturally satisfied by the implementation approach.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel (different files, no dependencies)
- **[Story]**: Which user story this task belongs to (e.g., US1, US2, US3)
- Include exact file paths in descriptions

---

## Phase 1: Setup

**Purpose**: Create new formatter crate and update workspace

- [x] T001 Add `crates/cgt-formatter-plain` to workspace members in `Cargo.toml`
- [x] T002 Create `crates/cgt-formatter-plain/Cargo.toml` with dependencies (cgt-core, rust_decimal, chrono)
- [x] T003 Create `crates/cgt-formatter-plain/src/lib.rs` with empty `format()` function signature
- [x] T004 Verify new crate compiles with `cargo build -p cgt-formatter-plain`

---

## Phase 2: Foundational (Blocking Prerequisites)

**Purpose**: Core formatter infrastructure needed before implementing output sections

**⚠️ CRITICAL**: No user story work can begin until this phase is complete

- [x] T005 Implement `get_exemption(year: u16) -> Decimal` function for UK annual exemption lookup in `crates/cgt-formatter-plain/src/lib.rs`
- [x] T006 Implement number formatting helpers (currency with £, decimal precision) in `crates/cgt-formatter-plain/src/lib.rs`
- [x] T007 Implement date formatting helper (DD/MM/YYYY format) in `crates/cgt-formatter-plain/src/lib.rs`

**Checkpoint**: Formatter crate has all helper functions - section implementation can begin

---

## Phase 3: User Story 1 - Generate Plain Text Tax Report (Priority: P1) 🎯 MVP

**Goal**: Produce human-readable plain text report matching cgtcalc format with all sections.

**Independent Test**: Run `cargo run -p cgt-cli -- report --year 2018 --format plain tests/data/Simple.cgt` and verify output matches expected format.

### Tests for User Story 1

> **NOTE: Create expected .txt files from cgtcalc reference, then implement formatter**

- [x] T008 [P] [US1] Create `tests/data/Simple.txt` expected output from cgtcalc reference
- [x] T009 [P] [US1] Create `tests/data/GainsAndLosses.txt` expected output from cgtcalc reference
- [x] T010 [P] [US1] Create `tests/data/HMRCExample1.txt` expected output from cgtcalc reference
- [x] T011 [P] [US1] Create `tests/data/MultipleMatches.txt` expected output (multiple tax years)

### Implementation for User Story 1

- [x] T012 [US1] Implement SUMMARY section formatter in `crates/cgt-formatter-plain/src/lib.rs`
- [x] T013 [US1] Implement TAX YEAR DETAILS section formatter in `crates/cgt-formatter-plain/src/lib.rs`
- [x] T014 [US1] Implement TAX RETURN INFORMATION section formatter in `crates/cgt-formatter-plain/src/lib.rs`
- [x] T015 [US1] Implement HOLDINGS section formatter in `crates/cgt-formatter-plain/src/lib.rs`
- [x] T016 [US1] Implement TRANSACTIONS section formatter in `crates/cgt-formatter-plain/src/lib.rs`
- [x] T017 [US1] Implement ASSET EVENTS section formatter in `crates/cgt-formatter-plain/src/lib.rs`
- [x] T018 [US1] Combine all sections in main `format()` function in `crates/cgt-formatter-plain/src/lib.rs`
- [x] T019 [US1] Run `cargo test -p cgt-formatter-plain` to verify formatter output

**Checkpoint**: User Story 1 complete - plain text formatter produces correct output for all test cases

---

## Phase 4: User Story 2 - Select Output Format via CLI (Priority: P2)

**Goal**: Add `--format` argument to CLI to switch between plain and JSON output.

**Independent Test**: Run CLI with `--format plain` and `--format json` and verify correct output types.

### Implementation for User Story 2

- [x] T020 [US2] Add `OutputFormat` enum (Plain, Json) to `crates/cgt-cli/src/commands.rs`
- [x] T021 [US2] Add `--format` argument with default `Plain` to Report command in `crates/cgt-cli/src/commands.rs`
- [x] T022 [US2] Add `cgt-formatter-plain` dependency to `crates/cgt-cli/Cargo.toml`
- [x] T023 [US2] Implement format dispatching in `crates/cgt-cli/src/main.rs` (plain vs json output)
- [x] T024 [US2] Verify `--format plain` produces plain text output
- [x] T025 [US2] Verify `--format json` produces JSON output (existing behavior)
- [x] T026 [US2] Verify default (no --format) produces plain text output

**Checkpoint**: User Story 2 complete - CLI supports format selection

---

## Phase 5: User Story 3 - Extensible Formatter Architecture (Priority: P3)

**Goal**: Verify the architecture is extensible - formatter is a separate crate.

**Independent Test**: Examine crate structure and verify formatter crate is independent.

### Verification for User Story 3

- [x] T027 [US3] Verify `cgt-formatter-plain` crate compiles independently with `cargo build -p cgt-formatter-plain`
- [x] T028 [US3] Verify `cgt-cli` imports formatter as dependency (check Cargo.toml)
- [x] T029 [US3] Verify formatter interface is simple: `format(&TaxReport, &[Transaction]) -> String`

**Checkpoint**: User Story 3 verified - architecture supports future formatters

---

## Phase 6: Polish & Cross-Cutting Concerns

**Purpose**: Final verification and cleanup

- [x] T030 Run full test suite: `cargo test` - verify all tests pass
- [x] T031 Run clippy: `cargo clippy` - verify no warnings
- [x] T032 Run fmt: `cargo fmt --check` - verify formatting
- [x] T033 Compare plain output numbers against cgtcalc reference for all test files
- [ ] T034 Commit changes with descriptive message

---

## Dependencies & Execution Order

### Phase Dependencies

- **Setup (Phase 1)**: No dependencies - create new crate
- **Foundational (Phase 2)**: Depends on Setup - BLOCKS user stories
- **User Story 1 (Phase 3)**: Depends on Foundational - core formatter implementation
- **User Story 2 (Phase 4)**: Depends on User Story 1 - CLI integration
- **User Story 3 (Phase 5)**: Verification only - can run after US1
- **Polish (Phase 6)**: Depends on all user stories

### User Story Dependencies

- **User Story 1 (P1)**: Core MVP - formatter implementation
- **User Story 2 (P2)**: Depends on US1 - needs formatter to dispatch to
- **User Story 3 (P3)**: Verification only - architecture is established by US1

### Within Each User Story

- Test files (.txt) created BEFORE formatter implementation
- Formatter sections implemented sequentially (SUMMARY → TAX YEAR DETAILS → ... → ASSET EVENTS)
- `cargo test` validates each story completion

### Parallel Opportunities

**During Phase 3 (User Story 1)**:

- T008, T009, T010, T011 (test file creation) can run in parallel

---

## Parallel Example: Test File Creation

```bash
# Launch all test file creation tasks together:
Task: "Create tests/data/Simple.txt expected output"
Task: "Create tests/data/GainsAndLosses.txt expected output"
Task: "Create tests/data/HMRCExample1.txt expected output"
Task: "Create tests/data/MultipleMatches.txt expected output"
```

---

## Implementation Strategy

### MVP First (User Story 1 Only)

1. Complete Phase 1: Setup (create crate)
2. Complete Phase 2: Foundational (helpers)
3. Complete Phase 3: User Story 1 (formatter sections)
4. **STOP and VALIDATE**: Test with `cargo test -p cgt-formatter-plain`
5. This is a working MVP with plain text output

### Incremental Delivery

1. Setup + Foundational → Crate ready
2. User Story 1 → Plain text formatter works → MVP!
3. User Story 2 → CLI integration → Full feature
4. User Story 3 → Architecture verified → Complete

### Single Developer Strategy

Execute phases sequentially:

1. Setup → Foundational → US1 → US2 → US3 → Polish

---

## Notes

- Test files MUST contain expected output matching cgtcalc format exactly
- Numerical values in output MUST match cgtcalc reference
- Minor spacing/formatting differences are acceptable if numbers match
- Run `cargo test` and `cargo clippy` after each phase
- The formatter needs both `TaxReport` and `Vec<Transaction>` (TRANSACTIONS section needs original data)
