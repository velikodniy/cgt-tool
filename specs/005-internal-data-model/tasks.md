# Tasks: Internal Data Model Improvements

**Input**: Design documents from `/specs/005-internal-data-model/`
**Prerequisites**: plan.md, spec.md, research.md, data-model.md, quickstart.md

**Tests**: Tests are NOT explicitly requested. Existing test infrastructure will be used to validate changes via test file migration.

**Organization**: Tasks are grouped by user story to enable independent implementation and testing of each story.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel (different files, no dependencies)
- **[Story]**: Which user story this task belongs to (e.g., US1, US2, US3)
- Include exact file paths in descriptions

## Path Conventions

This project uses a Rust workspace:

- **Core library**: `crates/cgt-core/src/`
- **CLI binary**: `crates/cgt-cli/src/`
- **Tests**: `tests/` (integration tests, test data)

---

## Phase 1: Setup (Shared Infrastructure)

**Purpose**: Add error variant and foundational types needed by all user stories

- [ ] T001 Add `InvalidTaxYear(u16)` error variant in `crates/cgt-core/src/error.rs`
- [ ] T002 Create `TaxPeriod` struct with validation in `crates/cgt-core/src/models.rs`
- [ ] T003 Implement custom `Serialize` for `TaxPeriod` (outputs "2023/24") in `crates/cgt-core/src/models.rs`
- [ ] T004 Implement custom `Deserialize` for `TaxPeriod` (validates consecutive years) in `crates/cgt-core/src/models.rs`
- [ ] T005 Implement `TaxPeriod::from_date()` for deriving tax year from any date in `crates/cgt-core/src/models.rs`
- [ ] T006 Add `JsonSchema` impl for `TaxPeriod` in `crates/cgt-core/src/models.rs`

---

## Phase 2: Foundational (Blocking Prerequisites)

**Purpose**: Core model restructuring that MUST be complete before ANY user story can be implemented

**⚠️ CRITICAL**: No user story work can begin until this phase is complete

- [ ] T007 Create `Disposal` struct (date, ticker, quantity, proceeds, matches) in `crates/cgt-core/src/models.rs`
- [ ] T008 Refactor `Match` struct: remove date/ticker, add `acquisition_date: Option<NaiveDate>` in `crates/cgt-core/src/models.rs`
- [ ] T009 Create `TaxYearSummary` struct (period, disposals, totals) in `crates/cgt-core/src/models.rs`
- [ ] T010 Refactor `TaxReport` struct: replace `tax_year: i32` and `matches` with `tax_years: Vec<TaxYearSummary>` in `crates/cgt-core/src/models.rs`
- [ ] T011 Update `crates/cgt-core/src/lib.rs` exports if needed for new types

**Checkpoint**: Model refactoring complete - calculator and tests can now be updated

---

## Phase 3: User Story 1 - Generate Human-Readable Tax Report (Priority: P1) 🎯 MVP

**Goal**: Data model supports generating clear tax reports with disposals grouped by tax year, showing sale details with matched acquisitions

**Independent Test**: Calculator produces output with `tax_years[].disposals[].matches[]` structure, and at least one test file validates correctly

### Implementation for User Story 1

- [ ] T012 [US1] Update calculator to group matches into `Disposal` objects (by date+ticker) in `crates/cgt-core/src/calculator.rs`
- [ ] T013 [US1] Update calculator to populate `Disposal.proceeds` from sell transaction in `crates/cgt-core/src/calculator.rs`
- [ ] T014 [US1] Update calculator to wrap disposals in `TaxYearSummary` using `TaxPeriod::from_date()` in `crates/cgt-core/src/calculator.rs`
- [ ] T015 [US1] Update calculator to compute per-year totals (total_gain, total_loss, net_gain) in `crates/cgt-core/src/calculator.rs`
- [ ] T016 [US1] Migrate `tests/data/Simple.json` to new format (single tax year, single disposal)
- [ ] T017 [US1] Run `cargo test` to verify Simple test passes
- [ ] T018 [P] [US1] Migrate `tests/data/SameDayMerge.json` to new format
- [ ] T019 [P] [US1] Migrate `tests/data/SameDayMergeInterleaved.json` to new format
- [ ] T020 [P] [US1] Migrate `tests/data/SimpleTwoSameDay.json` to new format
- [ ] T021 [P] [US1] Migrate `tests/data/GainsAndLosses.json` to new format

**Checkpoint**: Core tax report generation works with same-day matches

---

## Phase 4: User Story 2 - Understand Matching Logic (Priority: P2)

**Goal**: B&B matches include `acquisition_date` for audit trail, all matching rules properly attributed

**Independent Test**: B&B test file shows both disposal date and acquisition date in match output

### Implementation for User Story 2

- [ ] T022 [US2] Update calculator to populate `acquisition_date` for B&B matches in `crates/cgt-core/src/calculator.rs`
- [ ] T023 [US2] Migrate `tests/data/MultipleMatches.json` to new format (has B&B match)
- [ ] T024 [P] [US2] Migrate `tests/data/WithAssetEventsBB.json` to new format
- [ ] T025 [P] [US2] Migrate `tests/data/WithSplitBB.json` to new format
- [ ] T026 [P] [US2] Migrate `tests/data/WithUnsplitBB.json` to new format
- [ ] T027 [US2] Run `cargo test` to verify B&B tests pass with acquisition dates

**Checkpoint**: All matching rules display correctly with full audit trail

---

## Phase 5: User Story 3 - Multi-Year Portfolio View (Priority: P3)

**Goal**: Data model supports multiple tax years in single report, disposals correctly grouped by period

**Independent Test**: Test file with transactions spanning multiple tax years shows separate `TaxYearSummary` entries

### Implementation for User Story 3

- [ ] T028 [US3] Migrate `tests/data/CarryLoss.json` to new format (multi-year: 2017/18, 2018/19, 2019/20)
- [ ] T029 [P] [US3] Migrate `tests/data/WithAssetEventsMultipleYears.json` to new format
- [ ] T030 [P] [US3] Migrate `tests/data/2024_2025_SpecialYear.json` to new format
- [ ] T031 [US3] Verify multi-year totals are computed correctly per tax year in output

**Checkpoint**: Multi-year reports work with separate summaries per tax year

---

## Phase 6: Complete Test Migration

**Purpose**: Migrate remaining test files to new format

### Section 104 Pool Tests

- [ ] T032 [P] Migrate `tests/data/HMRCExample1.json` to new format
- [ ] T033 [P] Migrate `tests/data/WithSplitS104.json` to new format
- [ ] T034 [P] Migrate `tests/data/WithUnsplitS104.json` to new format

### Asset Event Tests

- [ ] T035 [P] Migrate `tests/data/WithAssetEvents.json` to new format
- [ ] T036 [P] Migrate `tests/data/WithAssetEventsSameDay.json` to new format
- [ ] T037 [P] Migrate `tests/data/AssetEventsNotFullSale.json` to new format
- [ ] T038 [P] Migrate `tests/data/AssetEventsNotFullSale2.json` to new format

### Capital Return and Special Cases

- [ ] T039 [P] Migrate `tests/data/BuySellAllBuyAgainCapitalReturn.json` to new format
- [ ] T040 [P] Migrate `tests/data/Blank.json` to new format (empty case)
- [ ] T041 [P] Migrate `tests/data/unsorted_transactions.json` to new format

**Checkpoint**: All 24 test files migrated

---

## Phase 7: Polish & Cross-Cutting Concerns

**Purpose**: Final validation and cleanup

- [ ] T042 Run `cargo test` - all tests must pass
- [ ] T043 Run `cargo clippy` - fix any warnings
- [ ] T044 Run `cargo fmt` - ensure consistent formatting
- [ ] T045 Update JSON schema if using `schemars` output
- [ ] T046 Run quickstart.md validation checklist

---

## Dependencies & Execution Order

### Phase Dependencies

- **Setup (Phase 1)**: No dependencies - can start immediately
- **Foundational (Phase 2)**: Depends on Setup completion - BLOCKS all user stories
- **User Stories (Phase 3-5)**: All depend on Foundational phase completion
  - US1 must complete first (establishes core calculator changes)
  - US2 can start after US1 (adds B&B acquisition date tracking)
  - US3 can start after US1 (adds multi-year grouping)
- **Test Migration (Phase 6)**: Depends on all calculator updates complete
- **Polish (Phase 7)**: Depends on all test files migrated

### User Story Dependencies

- **User Story 1 (P1)**: Can start after Foundational (Phase 2) - No dependencies on other stories
- **User Story 2 (P2)**: Depends on US1 calculator changes (T012-T015), adds B&B-specific logic
- **User Story 3 (P3)**: Depends on US1 calculator changes (T012-T015), validates multi-year grouping

### Within Each Phase

- Model changes before calculator changes
- Calculator changes before test file migration
- Core implementation before validation
- Commit after each task or logical group

### Parallel Opportunities

- Setup tasks T002-T006 touch same file but different functions (serialize sequentially)
- Phase 6 test migrations (T032-T041) are fully parallel (different files)
- Within Phase 3: T018-T021 are parallel (different test files)
- Within Phase 4: T024-T026 are parallel (different test files)
- Within Phase 5: T029-T030 are parallel (different test files)

---

## Parallel Example: Phase 6 Test Migration

```bash
# Launch all remaining test migrations together:
Task: "Migrate tests/data/HMRCExample1.json to new format"
Task: "Migrate tests/data/WithSplitS104.json to new format"
Task: "Migrate tests/data/WithUnsplitS104.json to new format"
Task: "Migrate tests/data/WithAssetEvents.json to new format"
Task: "Migrate tests/data/WithAssetEventsSameDay.json to new format"
Task: "Migrate tests/data/AssetEventsNotFullSale.json to new format"
Task: "Migrate tests/data/AssetEventsNotFullSale2.json to new format"
Task: "Migrate tests/data/BuySellAllBuyAgainCapitalReturn.json to new format"
Task: "Migrate tests/data/Blank.json to new format"
Task: "Migrate tests/data/unsorted_transactions.json to new format"
```

---

## Implementation Strategy

### MVP First (User Story 1 Only)

1. Complete Phase 1: Setup (TaxPeriod type)
2. Complete Phase 2: Foundational (model restructuring)
3. Complete Phase 3: User Story 1 (core calculator + 5 test files)
4. **STOP and VALIDATE**: `cargo test` passes for migrated files
5. Deploy/demo if ready - basic tax reports work

### Incremental Delivery

1. Complete Setup + Foundational → Model ready
2. Add User Story 1 → Basic reports work → Run tests (MVP!)
3. Add User Story 2 → B&B audit trail works → Run tests
4. Add User Story 3 → Multi-year reports work → Run tests
5. Complete remaining test migration → Full validation
6. Polish → Production ready

### Sequential Approach (Recommended)

This feature modifies core calculator logic, so sequential execution is safer:

1. T001-T006: Setup (TaxPeriod)
2. T007-T011: Foundational (model restructuring)
3. T012-T021: User Story 1 (core + 5 tests)
4. T022-T027: User Story 2 (B&B + 4 tests)
5. T028-T031: User Story 3 (multi-year + 3 tests)
6. T032-T041: Remaining tests (parallel safe)
7. T042-T046: Polish

---

## Notes

- [P] tasks = different files, no dependencies
- [Story] label maps task to specific user story for traceability
- Each user story should be independently completable and testable
- Verify `cargo test` passes after each user story phase
- Commit after each task or logical group
- Stop at any checkpoint to validate story independently
- Total tasks: 46
