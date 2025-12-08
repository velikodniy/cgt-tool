# Tasks: Strict Clippy Linting

**Input**: Design documents from `/specs/003-strict-clippy-linting/`
**Prerequisites**: plan.md, spec.md, research.md, data-model.md, quickstart.md

**Tests**: Not explicitly requested - existing tests will be maintained.

**Organization**: Tasks are grouped by user story to enable independent implementation and testing of each story.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel (different files, no dependencies)
- **[Story]**: Which user story this task belongs to (e.g., US1, US2, US3)
- Include exact file paths in descriptions

## Path Conventions

- **Workspace root**: `clippy.toml` (NEW)
- **Core library**: `crates/cgt-core/src/`
- **CLI binary**: `crates/cgt-cli/src/`
- **Core tests**: `crates/cgt-core/tests/`

---

## Phase 1: Setup (Clippy Configuration)

**Purpose**: Create centralized strict linting configuration

- [x] T001 Create clippy.toml at workspace root with deny-level lints (unwrap_used, expect_used, panic, todo, unimplemented) in /clippy.toml

**Checkpoint**: Running `cargo clippy` will now report all violations

---

## Phase 2: Foundational (Error Type Extensions)

**Purpose**: Extend CgtError with new variants needed for unwrap replacement

**⚠️ CRITICAL**: Error variants must exist before parser/calculator refactoring

- [x] T002 Add UnexpectedParserState variant to CgtError in crates/cgt-core/src/error.rs
- [x] T003 Add InvalidDateYear variant to CgtError in crates/cgt-core/src/error.rs

**Checkpoint**: New error types available for use in subsequent phases

---

## Phase 3: User Story 1 - Developer Builds Project with Strict Linting (Priority: P1) 🎯 MVP

**Goal**: Enforce strict linting with no suppressed warnings - `cargo clippy` passes cleanly

**Independent Test**: Run `cargo clippy --lib --bins -- -D warnings` and verify zero warnings

### Implementation for User Story 1

- [x] T004 [US1] Remove `#[allow(unused_imports)]` and fix underlying issue in crates/cgt-core/tests/parser_tests.rs
- [x] T005 [US1] Remove `#[allow(clippy::needless_range_loop)]` and refactor to use .iter().enumerate() in crates/cgt-core/src/calculator.rs
- [x] T006 [US1] Verify `cargo clippy --lib --bins` passes with zero warnings

**Checkpoint**: Strict linting enforced - no allow attributes in codebase

---

## Phase 4: User Story 2 - User Receives Meaningful Error for Malformed Input (Priority: P1)

**Goal**: Replace all .unwrap() in parser.rs with proper error handling that provides contextual messages

**Independent Test**: Parse malformed input files and verify error messages include context

### Implementation for User Story 2

- [x] T007 [US2] Replace .unwrap() on date_pair extraction with ok_or in crates/cgt-core/src/parser.rs
- [x] T008 [US2] Replace .unwrap() on command_pair extraction with ok_or in crates/cgt-core/src/parser.rs
- [x] T009 [US2] Replace .unwrap() on args extraction in parse_buy_sell with ok_or in crates/cgt-core/src/parser.rs
- [x] T010 [US2] Replace .unwrap() on ticker/amount/price extraction in parse_buy_sell with ok_or in crates/cgt-core/src/parser.rs
- [x] T011 [US2] Replace .unwrap() on args extraction in parse_dividend with ok_or in crates/cgt-core/src/parser.rs
- [x] T012 [US2] Replace .unwrap() on ticker/amount/total_value/tax_paid extraction in parse_dividend with ok_or in crates/cgt-core/src/parser.rs
- [x] T013 [US2] Replace .unwrap() on args extraction in parse_capreturn with ok_or in crates/cgt-core/src/parser.rs
- [x] T014 [US2] Replace .unwrap() on ticker/amount/total_value/expenses extraction in parse_capreturn with ok_or in crates/cgt-core/src/parser.rs
- [x] T015 [US2] Replace .unwrap() on args extraction in parse_split with ok_or in crates/cgt-core/src/parser.rs
- [x] T016 [US2] Replace .unwrap() on ticker/ratio extraction in parse_split with ok_or in crates/cgt-core/src/parser.rs
- [x] T017 [US2] Verify all parser functions return Result with descriptive errors
- [x] T018 [US2] Run `cargo test` to verify existing tests still pass

**Checkpoint**: Parser produces meaningful error messages instead of panicking

---

## Phase 5: User Story 3 - Developer Gets Helpful Errors During Development (Priority: P2)

**Goal**: Replace .unwrap() in calculator.rs with proper error handling for date creation

**Independent Test**: Trigger edge cases in calculator and verify errors contain context

### Implementation for User Story 3

- [x] T019 [US3] Replace .unwrap() on tax year start_date creation with ok_or returning InvalidDateYear in crates/cgt-core/src/calculator.rs
- [x] T020 [US3] Replace .unwrap() on tax year end_date creation with ok_or returning InvalidDateYear in crates/cgt-core/src/calculator.rs
- [x] T021 [US3] Verify calculator functions propagate errors correctly
- [x] T022 [US3] Run `cargo test` to verify existing tests still pass

**Checkpoint**: Calculator produces meaningful error messages for edge cases

---

## Phase 6: Test Code Updates

**Purpose**: Update test code to use .expect() with descriptive messages and add lint exemptions

- [x] T023 [P] Add `#![allow(clippy::expect_used, clippy::panic)]` to test module in crates/cgt-core/tests/parser_tests.rs
- [x] T024 [P] Replace all .unwrap() with .expect("descriptive message") in crates/cgt-core/tests/parser_tests.rs
- [x] T025 [P] Add `#![allow(clippy::expect_used)]` to test module in crates/cgt-core/tests/matching_tests.rs
- [x] T026 [P] Replace all .unwrap() with .expect("descriptive message") in crates/cgt-core/tests/matching_tests.rs

**Checkpoint**: Test code compiles and passes with strict linting

---

## Phase 7: Polish & Verification

**Purpose**: Final verification that all success criteria are met

- [x] T027 Run `cargo clippy --lib --bins -- -D warnings` and verify zero warnings
- [x] T028 Run `cargo clippy --tests -- -D warnings` and verify zero warnings (with exemptions)
- [x] T029 Run `cargo test` and verify 100% test pass rate
- [x] T030 Verify no .unwrap() in production code: `grep -r "\.unwrap()" crates/*/src/`
- [x] T031 Verify no #[allow(...)] in production code: `grep -r "#\[allow(" crates/*/src/`
- [x] T032 Run quickstart.md validation checklist

---

## Dependencies & Execution Order

### Phase Dependencies

- **Setup (Phase 1)**: No dependencies - can start immediately
- **Foundational (Phase 2)**: Depends on Setup - adds error types
- **User Story 1 (Phase 3)**: Depends on Phase 1 only - removes allow attributes
- **User Story 2 (Phase 4)**: Depends on Phases 1 and 2 - needs error types and clippy config
- **User Story 3 (Phase 5)**: Depends on Phases 1 and 2 - needs error types and clippy config
- **Test Updates (Phase 6)**: Depends on Phase 1 - needs clippy config
- **Polish (Phase 7)**: Depends on all previous phases

### User Story Dependencies

- **User Story 1 (P1)**: Independent - only removes allow attributes
- **User Story 2 (P1)**: Requires error types from Phase 2
- **User Story 3 (P2)**: Requires error types from Phase 2

### Within Each User Story

- Error types before implementation that uses them
- Production code before tests that verify it
- Verification after all changes in the story

### Parallel Opportunities

- T002 and T003 (error variants) can run in parallel
- T004 and T005 (allow removals) can run in parallel
- T023, T024, T025, T026 (test updates) can run in parallel
- User Stories 2 and 3 can run in parallel after Phase 2 completes

---

## Parallel Example: Test Updates (Phase 6)

```bash
# Launch all test file updates in parallel:
Task: "Add expect_used exemption and update parser_tests.rs"
Task: "Add expect_used exemption and update matching_tests.rs"
```

---

## Implementation Strategy

### MVP First (User Story 1 Only)

1. Complete Phase 1: Setup (clippy.toml)
2. Complete Phase 3: User Story 1 (remove allow attributes)
3. **STOP and VALIDATE**: Run `cargo clippy` - should pass with no warnings from allow attributes
4. Strict linting foundation established

### Incremental Delivery

1. Phase 1 + Phase 2 → Clippy config + error types ready
2. Add User Story 1 → No allow attributes → Validate
3. Add User Story 2 → Parser error handling → Validate
4. Add User Story 3 → Calculator error handling → Validate
5. Phase 6 → Test code updates → Validate
6. Phase 7 → Final verification → Complete

### Recommended Order (Single Developer)

1. T001 (clippy.toml)
2. T002, T003 (error types)
3. T004, T005 (remove allows)
4. T007-T018 (parser unwraps)
5. T019-T022 (calculator unwraps)
6. T023-T026 (test updates)
7. T027-T032 (verification)

---

## Notes

- [P] tasks = different files, no dependencies
- [Story] label maps task to specific user story for traceability
- Each user story should be independently completable and testable
- Commit after each task or logical group (per Commit Discipline in constitution)
- Stop at any checkpoint to validate story independently
- All .unwrap() replacements follow the ok_or pattern from research.md
