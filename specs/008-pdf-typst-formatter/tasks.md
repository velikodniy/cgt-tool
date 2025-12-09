# Tasks: PDF Typst Formatter

**Input**: Design documents from `/specs/008-pdf-typst-formatter/`
**Prerequisites**: plan.md, spec.md, research.md, data-model.md, quickstart.md

**Tests**: Integration tests included to verify PDF generation for all test cases.

**Organization**: Tasks grouped by user story. US1+US2 are P1 (core PDF with summary), US3+US4 are P2 (details), US5 is P3 (transactions).

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel (different files, no dependencies)
- **[Story]**: Which user story this task belongs to (e.g., US1, US2)
- All paths are relative to repository root

---

## Phase 1: Setup

**Purpose**: Create cgt-formatter-pdf crate and add dependencies

- [x] T001 Create crate directory structure at crates/cgt-formatter-pdf/
- [x] T002 Create Cargo.toml with typst-as-lib and typst-pdf dependencies in crates/cgt-formatter-pdf/Cargo.toml
- [x] T003 Add cgt-formatter-pdf to workspace members in Cargo.toml
- [x] T004 Add PdfGeneration error variant to crates/cgt-core/src/error.rs
- [x] T005 Verify project compiles with cargo build

---

## Phase 2: Foundational (Blocking Prerequisites)

**Purpose**: Core Typst infrastructure that all PDF sections depend on

**⚠️ CRITICAL**: No user story work can begin until this phase is complete

- [x] T006 Create lib.rs skeleton with public format() function signature in crates/cgt-formatter-pdf/src/lib.rs
- [x] T007 Implement Typst engine initialization with embedded fonts in crates/cgt-formatter-pdf/src/lib.rs
- [x] T008 Implement helper functions for currency and date formatting in crates/cgt-formatter-pdf/src/lib.rs
- [x] T009 Create basic Typst template with page setup (A4, margins, fonts) in crates/cgt-formatter-pdf/src/lib.rs
- [x] T010 Implement compile-to-PDF pipeline (Typst markup → compile → export) in crates/cgt-formatter-pdf/src/lib.rs
- [x] T011 Verify empty PDF generation works with cargo test

**Checkpoint**: Foundation ready - can now generate minimal valid PDFs

---

## Phase 3: User Story 1+2 - Generate PDF with Summary (Priority: P1) 🎯 MVP

**Goal**: Generate a valid PDF with header and summary table showing tax year, gains, proceeds, exemption, taxable gain

**Independent Test**: Run `cgt-cli report file.cgt --year 2023 --format pdf` and verify PDF opens in any reader with summary visible

### Implementation for User Stories 1+2

- [x] T012 [US1] Implement header generation (title, tax year, generation date) in crates/cgt-formatter-pdf/src/lib.rs
- [x] T013 [US2] Implement summary table generation with gain/loss, proceeds, exemption, taxable gain in crates/cgt-formatter-pdf/src/lib.rs
- [x] T014 [US1] Add Pdf variant to OutputFormat enum in crates/cgt-cli/src/commands.rs
- [x] T015 [US1] Add --output flag for PDF output path in crates/cgt-cli/src/commands.rs
- [x] T016 [US1] Implement PDF format handling in report command in crates/cgt-cli/src/main.rs
- [x] T017 [US1] Add cgt-formatter-pdf dependency to cgt-cli in crates/cgt-cli/Cargo.toml
- [x] T018 [US1] Manual test: generate PDF from tests/inputs/Simple.cgt and verify it opens

**Checkpoint**: MVP complete - users can generate valid PDFs with summary

---

## Phase 4: User Story 3 - Disposal Details (Priority: P2)

**Goal**: Add detailed disposal section showing each sale with matching rules applied

**Independent Test**: Generate PDF with multiple disposals, verify each shows shares, cost, proceeds, gain/loss and matching rule

### Implementation for User Story 3

- [x] T019 [US3] Implement disposal section heading in crates/cgt-formatter-pdf/src/lib.rs
- [x] T020 [US3] Implement disposal item formatting (ticker, date, quantity, gain/loss) in crates/cgt-formatter-pdf/src/lib.rs
- [x] T021 [US3] Implement match rule display (Same Day, B&B with date, Section 104 with cost/share) in crates/cgt-formatter-pdf/src/lib.rs
- [x] T022 [US3] Implement disposal calculation breakdown (proceeds, cost, result) in crates/cgt-formatter-pdf/src/lib.rs
- [x] T023 [US3] Manual test: generate PDF from tests/inputs/MultipleMatches.cgt and verify disposals display correctly

**Checkpoint**: Disposal details complete - users can see calculation breakdowns

---

## Phase 5: User Story 4 - Holdings Section (Priority: P2)

**Goal**: Add holdings section showing remaining positions with quantity and average cost

**Independent Test**: Generate PDF from file with remaining holdings, verify they appear with ticker, quantity, avg cost

### Implementation for User Story 4

- [x] T024 [P] [US4] Implement holdings section heading in crates/cgt-formatter-pdf/src/lib.rs
- [x] T025 [US4] Implement holdings table/list (ticker, quantity, avg cost) in crates/cgt-formatter-pdf/src/lib.rs
- [x] T026 [US4] Handle empty holdings case (show "None" or similar) in crates/cgt-formatter-pdf/src/lib.rs
- [x] T027 [US4] Manual test: generate PDF from tests/inputs/HMRCExample1.cgt and verify holdings section

**Checkpoint**: Holdings section complete

---

## Phase 6: User Story 5 - Transaction History (Priority: P3)

**Goal**: Add transaction listing showing all buys/sells chronologically

**Independent Test**: Generate PDF, verify all input transactions appear in date order with type, ticker, quantity, price, fees

### Implementation for User Story 5

- [x] T028 [P] [US5] Implement transactions section heading in crates/cgt-formatter-pdf/src/lib.rs
- [x] T029 [US5] Implement transaction table (date, type, ticker, quantity, price, fees) in crates/cgt-formatter-pdf/src/lib.rs
- [x] T030 [US5] Handle asset events (dividend, capreturn, split, unsplit) in transaction display in crates/cgt-formatter-pdf/src/lib.rs
- [x] T031 [US5] Ensure transactions are sorted by date then ticker in crates/cgt-formatter-pdf/src/lib.rs
- [x] T032 [US5] Manual test: generate PDF from tests/inputs/WithAssetEvents.cgt and verify all transactions

**Checkpoint**: All PDF sections complete

---

## Phase 7: Polish & Integration Tests

**Purpose**: Verify all test cases work, add integration tests

- [x] T033 [P] Add integration test verifying PDF generation for all 26 test cases in crates/cgt-cli/tests/cli_tests.rs
- [x] T034 [P] Add unit test verifying PDF starts with %PDF header in crates/cgt-formatter-pdf/src/lib.rs
- [x] T035 Run cargo clippy and fix any warnings
- [x] T036 Run cargo test and verify all tests pass
- [x] T037 Update README.md to document --format pdf option
- [x] T038 Test edge cases: empty disposals, long ticker names, large numbers

---

## Dependencies & Execution Order

### Phase Dependencies

- **Setup (Phase 1)**: No dependencies - can start immediately
- **Foundational (Phase 2)**: Depends on Setup completion - BLOCKS all user stories
- **User Stories 1+2 (Phase 3)**: Depends on Foundational - MVP
- **User Story 3 (Phase 4)**: Can start after Phase 3 (builds on template)
- **User Story 4 (Phase 5)**: Can start after Phase 3 (independent section)
- **User Story 5 (Phase 6)**: Can start after Phase 3 (independent section)
- **Polish (Phase 7)**: Depends on all user stories complete

### User Story Dependencies

- **User Stories 1+2 (P1)**: Core PDF + Summary - MUST complete first
- **User Story 3 (P2)**: Disposals - Can run parallel with US4/US5
- **User Story 4 (P2)**: Holdings - Can run parallel with US3/US5
- **User Story 5 (P3)**: Transactions - Can run parallel with US3/US4

### Parallel Opportunities

After Phase 3 (MVP) completes:

- US3 (Disposals), US4 (Holdings), US5 (Transactions) can all be implemented in parallel
- They add independent sections to the PDF template

---

## Parallel Example: Post-MVP Sections

```bash
# After MVP (Phase 3) completes, launch these in parallel:
Task: "Implement disposal section" (US3)
Task: "Implement holdings section" (US4)
Task: "Implement transactions section" (US5)
```

---

## Implementation Strategy

### MVP First (User Stories 1+2 Only)

1. Complete Phase 1: Setup crate
2. Complete Phase 2: Typst infrastructure
3. Complete Phase 3: Header + Summary
4. **STOP and VALIDATE**: Generate PDF, open in reader, verify summary
5. Deploy/demo if ready

### Incremental Delivery

1. Setup + Foundational → Can compile, empty PDF works
2. Add US1+US2 → Valid PDF with summary (MVP!)
3. Add US3 → Disposal details visible
4. Add US4 → Holdings visible
5. Add US5 → Full transaction history
6. Polish → All 26 tests pass

---

## Notes

- All formatting functions should reuse patterns from cgt-formatter-plain where applicable
- Typst template is generated programmatically (not external file)
- Test PDFs by opening in macOS Preview, checking they're valid
- Keep Typst markup simple - tables and headings are sufficient
- Sort all lists deterministically (date, then ticker) for reproducible output
