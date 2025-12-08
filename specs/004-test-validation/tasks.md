# Tasks: Test Suite Validation and UK CGT Tax Rules Documentation

**Input**: Design documents from `/specs/004-test-validation/`
**Prerequisites**: plan.md (required), spec.md (required), research.md, data-model.md, quickstart.md

**Note**: This feature is primarily documentation and manual verification work, not code implementation. Tests are NOT being added - we are verifying and documenting existing tests.

**Organization**: Tasks are grouped by user story to enable independent implementation and verification of each story.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel (different files, no dependencies)
- **[Story]**: Which user story this task belongs to (e.g., US1, US2, US3)
- Include exact file paths in descriptions

## Path Conventions

- **Documentation**: Project root (`TAX_RULES.md`, `README.md`)
- **Test files**: `tests/data/*.cgt`
- **Expected outputs**: `tests/data/*.json`
- **Spec files**: `specs/004-test-validation/`

---

## Phase 1: Setup (Shared Infrastructure)

**Purpose**: Ensure research is complete and templates are ready for verification work

- [x] T001 Verify research.md contains HMRC tax rules (Same Day, B&B, Section 104) in specs/004-test-validation/research.md
- [x] T002 Verify research.md contains cgtcalc comparison mapping table in specs/004-test-validation/research.md
- [x] T003 Verify research.md contains comment templates for .cgt files in specs/004-test-validation/research.md
- [x] T004 Verify quickstart.md contains verification workflow guide in specs/004-test-validation/quickstart.md

**Checkpoint**: Research and templates ready - documentation and verification work can begin

---

## Phase 2: User Story 1 - UK CGT Tax Rules Documentation (Priority: P1) 🎯 MVP

**Goal**: Create authoritative TAX_RULES.md document explaining UK CGT matching rules with worked examples and HMRC references

**Independent Test**: Verify TAX_RULES.md exists in project root with sections for Same Day, B&B, and Section 104 rules, each with at least 2 examples and HMRC references

### Implementation for User Story 1

- [x] T005 [US1] Create TAX_RULES.md with document header and overview section in /TAX_RULES.md
- [x] T006 [US1] Add Same Day Rule section with definition, HMRC reference (CG51560), and 2 worked examples in /TAX_RULES.md
- [x] T007 [US1] Add Bed & Breakfast Rule section with 30-day rule definition, HMRC reference (CG51560), and 2 worked examples in /TAX_RULES.md
- [x] T008 [US1] Add Section 104 Pooling section with average cost calculation, HMRC reference (CG51575), and 2 worked examples in /TAX_RULES.md
- [x] T009 [US1] Add Special Cases section covering stock splits, capital returns, and carried losses in /TAX_RULES.md
- [x] T010 [US1] Add Tax Year Boundaries section (April 6 to April 5) with examples in /TAX_RULES.md
- [x] T011 [US1] Add References section with links to HMRC Capital Gains Manual (CG51500-CG51600) in /TAX_RULES.md
- [x] T012 [US1] Review TAX_RULES.md for accuracy against HMRC guidance and completeness per SC-001

**Checkpoint**: TAX_RULES.md complete - foundational documentation for all verification work ready

---

## Phase 3: User Story 2 - Test Case Validation Against cgtcalc Reference (Priority: P1)

**Goal**: Compare our test suite with cgtcalc reference implementation and add README attribution

**Independent Test**: Verify all 21 cgtcalc tests are compared, mapping documented in research.md, and README contains attribution

### Implementation for User Story 2

- [x] T013 [US2] Document cgtcalc comparison completion status in specs/004-test-validation/research.md (verify 21/21 tests mapped)
- [x] T014 [US2] Verify line order differences (reverse chronological) are documented in specs/004-test-validation/research.md
- [x] T015 [US2] Verify DSL syntax differences (date format, @ separator, EXPENSES keyword) are documented in specs/004-test-validation/research.md
- [x] T016 [US2] Add Attribution section to README.md acknowledging cgtcalc project for test case inspiration in /README.md
- [x] T017 [US2] Run cargo test to verify all tests pass before verification work

**Checkpoint**: cgtcalc comparison complete, attribution added - ready for manual verification

---

## Phase 4: User Story 3 - Manual Test Case Verification (Priority: P2)

**Goal**: Manually verify all 22 test cases by calculating expected gain/loss using TAX_RULES.md and document results

**Independent Test**: Each .cgt file has verification comments, 100% of test cases manually verified with calculations documented

### Priority Tests - Detailed Step-by-Step Verification (per SC-009)

These 6 complex/representative tests require detailed step-by-step verification comments:

- [x] T018 [US3] Add detailed step-by-step verification to tests/data/HMRCExample1.cgt (official HMRC example, authoritative)
- [x] T019 [US3] Add detailed step-by-step verification to tests/data/WithAssetEventsBB.cgt (B&B with asset events, multi-year)
- [x] T020 [US3] Add detailed step-by-step verification to tests/data/WithAssetEventsMultipleYears.cgt (most complex, 10 transactions, multi-year)
- [x] T021 [US3] Add detailed step-by-step verification to tests/data/MultipleMatches.cgt (demonstrates all three rules)
- [x] T022 [US3] Add detailed step-by-step verification to tests/data/SameDayMerge.cgt (core Same Day logic with merging)
- [x] T023 [US3] Add detailed step-by-step verification to tests/data/CarryLoss.cgt (loss carryover across years)

### Simple Tests - Verification with Header Comments

Same Day rule tests:

- [x] T024 [P] [US3] Add header and verification comments to tests/data/Simple.cgt (basic same day match)
- [x] T025 [P] [US3] Add header and verification comments to tests/data/SimpleTwoSameDay.cgt (two same day transactions)
- [x] T026 [P] [US3] Add header and verification comments to tests/data/SameDayMergeInterleaved.cgt (interleaved same day)
- [x] T027 [P] [US3] Add header and verification comments to tests/data/GainsAndLosses.cgt (gains and losses)

Section 104 pool tests:

- [x] T028 [P] [US3] Add header and verification comments to tests/data/WithSplitS104.cgt (split with S104)
- [x] T029 [P] [US3] Add header and verification comments to tests/data/WithUnsplitS104.cgt (unsplit with S104)

Bed & Breakfast tests:

- [x] T030 [P] [US3] Add header and verification comments to tests/data/WithSplitBB.cgt (split with B&B)
- [x] T031 [P] [US3] Add header and verification comments to tests/data/WithUnsplitBB.cgt (unsplit with B&B)

Asset event tests:

- [x] T032 [P] [US3] Add header and verification comments to tests/data/WithAssetEvents.cgt (basic asset events)
- [x] T033 [P] [US3] Add header and verification comments to tests/data/WithAssetEventsSameDay.cgt (asset events same day)
- [x] T034 [P] [US3] Add header and verification comments to tests/data/AssetEventsNotFullSale.cgt (partial sale with events)
- [x] T035 [P] [US3] Add header and verification comments to tests/data/AssetEventsNotFullSale2.cgt (partial sale variant)
- [x] T036 [P] [US3] Add header and verification comments to tests/data/BuySellAllBuyAgainCapitalReturn.cgt (capital return scenario)

Special/Edge case tests:

- [x] T037 [P] [US3] Add header and verification comments to tests/data/2024_2025_SpecialYear.cgt (tax year boundary)
- [x] T038 [P] [US3] Add header and verification comments to tests/data/unsorted_transactions.cgt (parser ordering test)
- [x] T039 [P] [US3] Add header and verification comments to tests/data/Blank.cgt (empty file test)

### Verification Validation

- [x] T040 [US3] Run cargo test to verify all tests still pass after adding comments
- [x] T041 [US3] Review all .cgt files to ensure 100% have header comments (SC-003)
- [x] T042 [US3] Review all .cgt files to ensure verification notes are present (SC-004)

**Checkpoint**: All 22 test cases manually verified with calculations documented in comments

---

## Phase 5: User Story 4 - Self-Documenting Test Files (Priority: P2)

**Goal**: Ensure all test files are self-documenting with comments explaining what each test validates

**Independent Test**: Each .cgt file has clear header comments explaining test purpose, rules tested, and expected behavior

### Implementation for User Story 4

Note: Header comments were added in Phase 4 (US3). This phase focuses on quality review and enhancement.

- [x] T043 [US4] Review all .cgt file headers for completeness: test name, purpose, rules tested, complexity, key features
- [x] T044 [US4] Verify complex test cases have inline comments explaining key transactions and matching logic per FR-010
- [x] T045 [US4] Verify multi-step tests document expected calculation flow (Same Day → B&B → Section 104)
- [x] T046 [US4] Update any insufficient comments identified during review

**Checkpoint**: All test files are self-documenting with comprehensive comments

---

## Phase 6: User Story 5 - Extended Test Coverage (Priority: P3)

**Goal**: Identify gaps in cgtcalc coverage and add test cases for scenarios beyond cgtcalc

**Independent Test**: Coverage gaps documented, any new test cases added have verified correct results

### Implementation for User Story 5

- [x] T047 [US5] Document coverage analysis in specs/004-test-validation/research.md (unsorted_transactions.cgt is only cgt-tool exclusive test)
- [x] T048 [US5] Review if additional edge cases are needed beyond current 22 tests
- [x] T049 [US5] If new tests needed: Create test case with manually verified expected output
- [x] T050 [US5] Run cargo test to verify all tests pass including any new tests

**Checkpoint**: Test coverage is comprehensive, gaps addressed

---

## Phase 7: Polish & Cross-Cutting Concerns

**Purpose**: Final validation and cleanup

- [x] T051 Verify SC-001: TAX_RULES.md has 3 rules with 2+ examples each
- [x] T052 Verify SC-002: cgtcalc comparison documented with commit hash 896d91486805e27fcea0e851ee01868b86e161f5
- [x] T053 Verify SC-003: 100% of .cgt files (22 files) have header comments
- [x] T054 Verify SC-004: 100% of test cases have verification notes in comments
- [x] T055 Verify SC-005: Any discrepancies documented with HMRC-based resolution
- [x] T056 Verify SC-006: All tests pass (cargo test)
- [x] T057 Verify SC-007: README.md has cgtcalc attribution
- [x] T058 Verify SC-009: 6 complex tests have detailed step-by-step verification
- [x] T059 Update spec status from Draft to Complete in specs/004-test-validation/spec.md
- [x] T060 Final review of all documentation for consistency and accuracy

---

## Dependencies & Execution Order

### Phase Dependencies

- **Setup (Phase 1)**: No dependencies - verify prerequisites
- **User Story 1 (Phase 2)**: Depends on Setup - creates foundational TAX_RULES.md
- **User Story 2 (Phase 3)**: Can start after Setup, benefits from US1 completion
- **User Story 3 (Phase 4)**: Depends on US1 (needs TAX_RULES.md for verification)
- **User Story 4 (Phase 5)**: Depends on US3 (reviews comments added in US3)
- **User Story 5 (Phase 6)**: Depends on US3 (needs verification complete to assess gaps)
- **Polish (Phase 7)**: Depends on all user stories complete

### User Story Dependencies

```
US1 (Tax Rules Doc) ───────────────────────────────────┐
        ↓                                              │
US2 (cgtcalc Comparison) ───┬──────────────────────────┤
        ↓                   │                          │
US3 (Manual Verification) ──┴──────────────────────────┤
        ↓                                              │
US4 (Self-Documenting) ────────────────────────────────┤
        ↓                                              │
US5 (Extended Coverage) ───────────────────────────────┤
        ↓                                              │
Polish ←───────────────────────────────────────────────┘
```

### Within Each User Story

- Documentation tasks before verification tasks
- Simple tasks before complex tasks (for learning)
- Priority tests (detailed verification) can be parallelized
- Simple tests marked [P] can all run in parallel

### Parallel Opportunities

**Within User Story 3 (Manual Verification):**

- All 6 priority detailed verification tasks (T018-T023) can run in parallel
- All 16 simple test comment tasks (T024-T039) can run in parallel
- Note: Different team members could work on different test files simultaneously

**Across User Stories:**

- US1 and US2 can start in parallel after Setup
- US3 depends on US1 but not US2 (though US2 completion is helpful)

---

## Parallel Example: User Story 3 Verification

```bash
# Launch all detailed verification tasks in parallel:
Task: "Add detailed step-by-step verification to tests/data/HMRCExample1.cgt"
Task: "Add detailed step-by-step verification to tests/data/WithAssetEventsBB.cgt"
Task: "Add detailed step-by-step verification to tests/data/WithAssetEventsMultipleYears.cgt"
Task: "Add detailed step-by-step verification to tests/data/MultipleMatches.cgt"
Task: "Add detailed step-by-step verification to tests/data/SameDayMerge.cgt"
Task: "Add detailed step-by-step verification to tests/data/CarryLoss.cgt"

# Launch all simple test comment tasks in parallel:
Task: "Add header and verification comments to tests/data/Simple.cgt"
Task: "Add header and verification comments to tests/data/SimpleTwoSameDay.cgt"
# ... etc for all 16 simple tests
```

---

## Implementation Strategy

### MVP First (User Story 1 Only)

1. Complete Phase 1: Setup (verify prerequisites)
2. Complete Phase 2: User Story 1 (TAX_RULES.md)
3. **STOP and VALIDATE**: TAX_RULES.md covers all three rules with examples
4. This provides foundational documentation for all future work

### Incremental Delivery

1. **US1 Complete** → TAX_RULES.md ready (foundation for verification)
2. **US2 Complete** → cgtcalc comparison documented, README attribution added
3. **US3 Complete** → All 22 tests manually verified with comments
4. **US4 Complete** → Comment quality reviewed and enhanced
5. **US5 Complete** → Coverage gaps assessed and addressed
6. **Polish Complete** → All success criteria validated

### Recommended Approach

Since this is documentation/verification work (not code):

1. **Single implementer path**: US1 → US2 → US3 → US4 → US5 → Polish
2. **Focus on US3**: This is the bulk of the work (22 test files to verify)
3. **Parallel US3 work**: Different people can verify different test files
4. **Quality checkpoints**: Run `cargo test` after each test file modification

---

## Notes

- [P] tasks = different files, no dependencies
- [Story] label maps task to specific user story
- This feature is primarily documentation - no new code unless bugs found
- Manual verification follows templates in research.md
- HMRC guidance is authoritative for resolving discrepancies
- Do NOT change test expectations without HMRC proof (Constitution Principle III)
- Commit after each logical group of test file comments
- Run `cargo test` frequently to ensure tests still pass

---

## Summary

**Total Tasks**: 60

**Tasks per User Story**:

- Setup: 4 tasks
- US1 (Tax Rules Documentation): 8 tasks
- US2 (cgtcalc Comparison): 5 tasks
- US3 (Manual Verification): 25 tasks (6 detailed + 16 simple + 3 validation)
- US4 (Self-Documenting): 4 tasks
- US5 (Extended Coverage): 4 tasks
- Polish: 10 tasks

**Parallel Opportunities**:

- 6 detailed verification tasks in US3 can run in parallel
- 16 simple test comment tasks in US3 can run in parallel
- Total of 22 parallelizable tasks in US3 alone

**Independent Test Criteria per Story**:

- US1: TAX_RULES.md exists with 3 rules, 2+ examples each, HMRC references
- US2: cgtcalc mapping complete (21/21), README attribution present
- US3: 100% of .cgt files have verification comments
- US4: All comments reviewed for completeness and quality
- US5: Coverage gaps documented, any new tests verified

**Suggested MVP Scope**: User Story 1 (TAX_RULES.md) - provides foundational documentation

**Format Validation**: ✅ All tasks follow checklist format (checkbox, ID, labels, file paths)
