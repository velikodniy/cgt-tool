# Tasks: Multi-Ticker Support

**Input**: Design documents from `/specs/006-multi-ticker/`
**Prerequisites**: plan.md, spec.md, research.md, data-model.md, quickstart.md

**Tests**: Required - spec explicitly states "tests MUST contain manual calculations like other tests"

**Organization**: Tasks grouped by user story. The split-process-merge refactor (US1) enables all other stories automatically.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel (different files, no dependencies)
- **[Story]**: Which user story this task belongs to (e.g., US1, US2, US3, US4)
- Include exact file paths in descriptions

---

## Phase 1: Setup

**Purpose**: No new setup needed - existing Rust workspace structure is sufficient

- [x] T001 Verify existing tests pass with `cargo test`
- [x] T002 Verify clippy passes with `cargo clippy`

---

## Phase 2: Foundational (Blocking Prerequisites)

**Purpose**: Ticker normalization must be done first - required by all user stories

**⚠️ CRITICAL**: No user story work can begin until this phase is complete

- [x] T003 Add ticker uppercase normalization in `crates/cgt-core/src/parser.rs` (FR-009)
- [x] T004 Verify existing tests still pass after normalization change

**Checkpoint**: Parser now normalizes all tickers to uppercase - calculator refactor can begin

---

## Phase 3: User Story 1 - Calculate CGT for Portfolio with Multiple Stocks (Priority: P1) 🎯 MVP

**Goal**: Enable the calculator to handle transactions with multiple ticker symbols, maintaining separate Section 104 pools for each ticker.

**Independent Test**: Create a .cgt file with transactions for multiple tickers (AAPL, MSFT) and verify each has its own pool.

### Tests for User Story 1

> **NOTE: Write test file FIRST with manual calculations, then implement**

- [x] T005 [US1] Create `tests/data/MultiTickerBasic.cgt` with manual calculations in comments
- [x] T006 [US1] Create `tests/data/MultiTickerBasic.json` with expected output

### Implementation for User Story 1

- [x] T007 [US1] Refactor `calculate()` in `crates/cgt-core/src/calculator.rs` to group transactions by ticker first
- [x] T008 [US1] Extract single-ticker processing logic into helper function in `crates/cgt-core/src/calculator.rs`
- [x] T009 [US1] Implement result merging (combine disposals, collect pools) in `crates/cgt-core/src/calculator.rs`
- [x] T010 [US1] Change pool from `Option<Section104Holding>` to `HashMap<String, Section104Holding>` in `crates/cgt-core/src/calculator.rs`
- [x] T011 [US1] Run `cargo test` to verify MultiTickerBasic test passes
- [x] T012 [US1] Run `cargo clippy` to ensure no linting violations

**Checkpoint**: User Story 1 complete - multiple tickers with Section 104 pooling works independently

---

## Phase 4: User Story 2 - Same Day Matching with Multiple Tickers (Priority: P2)

**Goal**: Verify that Same Day matching only matches transactions of the same ticker.

**Independent Test**: Create same-day buy/sell pairs for different tickers and verify no cross-matching.

### Tests for User Story 2

- [x] T013 [US2] Create `tests/data/MultiTickerSameDay.cgt` with manual calculations in comments
- [x] T014 [US2] Create `tests/data/MultiTickerSameDay.json` with expected output

### Verification for User Story 2

- [x] T015 [US2] Run `cargo test` to verify MultiTickerSameDay test passes

**Checkpoint**: User Story 2 verified - Same Day rule respects ticker boundaries

---

## Phase 5: User Story 3 - Bed & Breakfast Matching with Multiple Tickers (Priority: P2)

**Goal**: Verify that B&B matching only matches transactions of the same ticker.

**Independent Test**: Create B&B scenarios with different tickers and verify no cross-matching.

### Tests for User Story 3

- [x] T016 [US3] Create `tests/data/MultiTickerBedAndBreakfast.cgt` with manual calculations in comments
- [x] T017 [US3] Create `tests/data/MultiTickerBedAndBreakfast.json` with expected output

### Verification for User Story 3

- [x] T018 [US3] Run `cargo test` to verify MultiTickerBedAndBreakfast test passes

**Checkpoint**: User Story 3 verified - B&B rule respects ticker boundaries

---

## Phase 6: User Story 4 - Stock Splits/Consolidations Per Ticker (Priority: P3)

**Goal**: Verify that SPLIT/UNSPLIT operations only affect the specified ticker's pool.

**Independent Test**: Have a split for one ticker and verify other tickers' pools are unaffected.

### Tests for User Story 4

- [x] T019 [US4] Create `tests/data/MultiTickerSplit.cgt` with manual calculations in comments
- [x] T020 [US4] Create `tests/data/MultiTickerSplit.json` with expected output

### Verification for User Story 4

- [x] T021 [US4] Run `cargo test` to verify MultiTickerSplit test passes

**Checkpoint**: User Story 4 verified - Split/Unsplit only affects specified ticker

---

## Phase 7: Polish & Cross-Cutting Concerns

**Purpose**: Final verification and cleanup

- [x] T022 Run full test suite: `cargo test` - verify all 22+ tests pass (SC-001)
- [x] T023 Run clippy: `cargo clippy` - verify no warnings (SC-005)
- [x] T024 Run fmt: `cargo fmt --check` - verify formatting
- [x] T025 Verify holdings output shows multiple tickers correctly (SC-004)
- [x] T026 Commit changes with descriptive message

---

## Dependencies & Execution Order

### Phase Dependencies

- **Setup (Phase 1)**: No dependencies - verify baseline
- **Foundational (Phase 2)**: Depends on Setup - BLOCKS all user stories
- **User Story 1 (Phase 3)**: Depends on Foundational - core implementation
- **User Stories 2-4 (Phases 4-6)**: Depend on User Story 1 completion
- **Polish (Phase 7)**: Depends on all user stories

### User Story Dependencies

- **User Story 1 (P1)**: Core refactor - must complete first, provides foundation for all others
- **User Story 2 (P2)**: Verification only - depends on US1 implementation
- **User Story 3 (P2)**: Verification only - depends on US1 implementation
- **User Story 4 (P3)**: Verification only - depends on US1 implementation

### Within Each User Story

- Test files (.cgt, .json) created BEFORE implementation verification
- Manual calculations in comments REQUIRED for each test
- `cargo test` validates each story completion

### Parallel Opportunities

**After Foundational (Phase 2) completes**:

- T005 and T006 (US1 test files) can be created in parallel

**After US1 Implementation (Phase 3) completes**:

- All test file creation tasks (T013-T014, T016-T017, T019-T020) can run in parallel
- Different team members can work on US2, US3, US4 tests simultaneously

---

## Parallel Example: Test File Creation

```bash
# After US1 implementation is complete, create all remaining test files in parallel:
Task: "Create tests/data/MultiTickerSameDay.cgt with manual calculations"
Task: "Create tests/data/MultiTickerBedAndBreakfast.cgt with manual calculations"
Task: "Create tests/data/MultiTickerSplit.cgt with manual calculations"
```

---

## Implementation Strategy

### MVP First (User Story 1 Only)

1. Complete Phase 1: Setup verification
2. Complete Phase 2: Foundational (ticker normalization)
3. Complete Phase 3: User Story 1 (calculator refactor + basic test)
4. **STOP and VALIDATE**: Test with `cargo test`
5. This is a working MVP with multi-ticker support

### Incremental Delivery

1. Setup + Foundational → Parser ready
2. User Story 1 → Core multi-ticker works → MVP!
3. User Story 2 → Same Day verified → More confidence
4. User Story 3 → B&B verified → More confidence
5. User Story 4 → Splits verified → Feature complete

### Single Developer Strategy

Execute phases sequentially:

1. Setup → Foundational → US1 → US2 → US3 → US4 → Polish

---

## Notes

- All test files MUST contain manual calculations in comments (Constitution Principle VI)
- Existing tests MUST NOT be modified (Constitution Principle III)
- Each task should be a single commit
- Run `cargo test` and `cargo clippy` after each phase
- The split-process-merge refactor in US1 automatically enables US2, US3, US4
