# Tasks: Better Testing Coverage

**Input**: Design documents from `/specs/010-better-testing/`
**Prerequisites**: plan.md (required), spec.md (required for user stories), research.md, data-model.md

**Tests**: This feature is test-focused; all tasks add test fixtures and assertions.

**Organization**: Tasks are grouped by user story to enable independent implementation and testing of each story.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel (different files, no dependencies)
- **[Story]**: Which user story this task belongs to (e.g., US1, US2, US3)
- Include exact file paths in descriptions

## Path Conventions

Based on plan.md structure:

- Test fixtures: `tests/inputs/`
- Expected JSON outputs: `tests/json/`
- Expected plain-text outputs: `tests/plain/`
- Crate integration tests: `crates/cgt-core/tests/`

---

## Phase 1: Setup

**Purpose**: Verify existing test infrastructure and identify any missing harness components

- [x] T001 Run `cargo test` to confirm existing tests pass before adding new fixtures
- [x] T002 [P] Review existing fixture structure in tests/inputs/, tests/json/, tests/plain/ to establish naming/format conventions

---

## Phase 2: Foundational (Research Validation)

**Purpose**: Confirm HMRC rules and expected calculations before writing fixtures

**⚠️ CRITICAL**: Expected outputs must be manually verified against HMRC rules before tests are added

- [x] T003 Document expected 2024/25 rate split calculations (pre-30 Oct: 10/20%; post-30 Oct: 18/24%) with worked examples in specs/010-better-testing/research.md
- [x] T004 Document expected accumulation dividend pool adjustments with worked example (only adjust holdings present on dividend date)
- [x] T005 Document expected CAPRETURN pool cost reduction with worked example (lump-sum, no share count)
- [x] T006 Document expected expenses/stamp duty treatment (increase allowable costs, exclude from proceeds) with worked example

**Checkpoint**: Research validated; fixtures can now be created with confidence in expected outputs

---

## Phase 3: User Story 1 - Validate 2024/25 rate split (Priority: P1) 🎯 MVP

**Goal**: Add regression tests confirming disposals before 30 Oct 2024 use old rates and disposals on/after use new rates

**Independent Test**: Run `cargo test` with the new fixture and confirm tax-year summary shows correct rate bands

### Implementation for User Story 1

- [x] T007 [P] [US1] Create fixture RateSplit2024.cgt in tests/inputs/ with disposals on 29 Oct 2024 and 30 Oct 2024
- [x] T008 [P] [US1] Manually compute expected gains for RateSplit2024 per HMRC rules; create tests/json/RateSplit2024.json
- [x] T009 [P] [US1] Manually compute expected plain-text output; create tests/plain/RateSplit2024.txt
- [x] T010 [US1] Run cargo test to verify RateSplit2024 fixture produces expected outputs; document any failures

**Checkpoint**: User Story 1 complete; 2024/25 rate split is regression-tested

---

## Phase 4: User Story 2 - Preserve correct pools and gains with dividends, equalisation, expenses (Priority: P2)

**Goal**: Add regression tests for accumulation dividends, CAPRETURN, and expenses/rounding

**Independent Test**: Run `cargo test` with new fixtures and confirm gains match hand-calculated expected values

### Implementation for User Story 2

#### Accumulation Dividends

- [x] T011 [P] [US2] Create fixture AccumulationDividend.cgt in tests/inputs/ with dividend after partial disposal
- [x] T012 [P] [US2] Manually compute expected gains (only adjust holdings on dividend date); create tests/json/AccumulationDividend.json
- [x] T013 [P] [US2] Create tests/plain/AccumulationDividend.txt with expected text output

#### Dividend After Full Disposal

- [x] T014 [P] [US2] Create fixture DividendAfterFullDisposal.cgt in tests/inputs/ with dividend when holdings are zero
- [x] T015 [P] [US2] Compute expected outputs (no pool change, no error); create tests/json/DividendAfterFullDisposal.json and tests/plain/DividendAfterFullDisposal.txt

#### CAPRETURN Equalisation

- [x] T016 [P] [US2] Create fixture CapReturnEqualisation.cgt in tests/inputs/ with CAPRETURN payment (no share count)
- [x] T017 [P] [US2] Manually compute expected pool cost reduction; create tests/json/CapReturnEqualisation.json and tests/plain/CapReturnEqualisation.txt

#### Expenses and Rounding

- [x] T018 [P] [US2] Create fixture ExpensesRounding.cgt in tests/inputs/ with buys/sells including fees and stamp duty

- [x] T019 [P] [US2] Manually compute expected allowable costs and proceeds; create tests/json/ExpensesRounding.json and tests/plain/ExpensesRounding.txt

- [x] T020 [US2] Run cargo test to verify all US2 fixtures produce expected outputs; document any failures

**Checkpoint**: User Story 2 complete; dividends, CAPRETURN, expenses regression-tested

---

## Phase 5: User Story 3 - Reporting clarity in text output (Priority: P3)

**Goal**: Add regression test for B&B matched quantity reporting and text output consistency

**Independent Test**: Run `cargo test` with new fixture and inspect plain-text output for correct matched quantities

### Implementation for User Story 3

- [x] T021 [P] [US3] Create fixture BnBReportQuantity.cgt in tests/inputs/ with B&B match scenario
- [x] T022 [P] [US3] Manually compute expected matched quantities and gains; create tests/json/BnBReportQuantity.json
- [x] T023 [P] [US3] Create tests/plain/BnBReportQuantity.txt ensuring B&B narrative shows matched quantity (not full pool)
- [x] T024 [US3] Run cargo test to verify BnBReportQuantity fixture; confirm text output matches calculations

**Checkpoint**: User Story 3 complete; B&B reporting clarity regression-tested

---

## Phase 6: Edge Cases & Guardrails

**Purpose**: Add tests for edge cases identified in spec

### FX Guardrail

- [N/A] T025 [P] Create fixture NonGBPCurrency.cgt in tests/inputs/ with USD-coded amount
  - **Note**: DSL does not support currency codes; parser inherently rejects non-numeric prices
- [x] T026 [P] Document expected error/warning message when non-GBP detected; create expected output asserting clear rejection
  - **Note**: Documented in research.md - parse error occurs for non-numeric values

### Whitespace Parsing

- [x] T027 [P] Create fixture WhitespaceDividend.cgt in tests/inputs/ with tabs and extra spaces in DIVIDEND line

- [x] T028 [P] Manually verify parsing handles whitespace; create tests/json/WhitespaceDividend.json and tests/plain/WhitespaceDividend.txt

- [x] T029 Run cargo test to verify edge case fixtures; document any failures

---

## Phase 7: Polish & Documentation

**Purpose**: Ensure all tests pass and document fixtures for maintenance

- [x] T030 Run full `cargo test` suite and confirm all new fixtures pass
- [x] T031 Run `cargo clippy` and fix any warnings in test code
- [x] T032 [P] Update specs/010-better-testing/quickstart.md with instructions for running new fixtures
- [x] T033 [P] Add inline comments in new fixture files explaining test purpose and HMRC rule reference

---

## Dependencies & Execution Order

### Phase Dependencies

- **Setup (Phase 1)**: No dependencies - can start immediately
- **Foundational (Phase 2)**: Depends on Setup; BLOCKS all user stories (expected outputs must be validated first)
- **User Stories (Phase 3-5)**: All depend on Foundational phase completion
  - User stories can proceed in parallel if desired
  - Or sequentially in priority order (P1 → P2 → P3)
- **Edge Cases (Phase 6)**: Can run in parallel with or after user stories
- **Polish (Phase 7)**: Depends on all fixture tasks being complete

### User Story Dependencies

- **User Story 1 (P1)**: Can start after Foundational (Phase 2) - No dependencies on other stories
- **User Story 2 (P2)**: Can start after Foundational (Phase 2) - Independent of US1
- **User Story 3 (P3)**: Can start after Foundational (Phase 2) - Independent of US1/US2

### Within Each User Story

- Fixture creation tasks (marked [P]) can run in parallel
- Final verification task (run cargo test) depends on all fixture tasks in that story

### Parallel Opportunities

- All Setup tasks marked [P] can run in parallel
- All fixture-creation tasks within a story marked [P] can run in parallel
- Different user stories can be worked on in parallel by different team members
- Edge case tasks marked [P] can run in parallel with user story tasks

---

## Parallel Example: User Story 2

```bash
# Launch all fixture-creation tasks together:
Task: "Create fixture AccumulationDividend.cgt in tests/inputs/"
Task: "Create fixture DividendAfterFullDisposal.cgt in tests/inputs/"
Task: "Create fixture CapReturnEqualisation.cgt in tests/inputs/"
Task: "Create fixture ExpensesRounding.cgt in tests/inputs/"

# Then run verification:
Task: "Run cargo test to verify all US2 fixtures"
```

---

## Implementation Strategy

### MVP First (User Story 1 Only)

1. Complete Phase 1: Setup
2. Complete Phase 2: Foundational (validate expected calculations)
3. Complete Phase 3: User Story 1 (rate split test)
4. **STOP and VALIDATE**: Run `cargo test` and confirm rate split works
5. Can deliver/demo at this point

### Incremental Delivery

1. Complete Setup + Foundational → Research validated
2. Add User Story 1 → Test independently → Rate split covered (MVP!)
3. Add User Story 2 → Test independently → Dividends/CAPRETURN/expenses covered
4. Add User Story 3 → Test independently → Reporting clarity covered
5. Add Edge Cases → FX guardrail and whitespace covered
6. Polish → Full suite passes, documentation complete

---

## Notes

- [P] tasks = different files, no dependencies
- [Story] label maps task to specific user story for traceability
- Each user story should be independently completable and testable
- Expected outputs MUST be manually computed per HMRC rules before fixtures are added
- Commit after each task or logical group
- Stop at any checkpoint to validate story independently
