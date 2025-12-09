# Tasks: Codebase Quality Refactoring

**Input**: Design documents from `/specs/009-codebase-refactoring/`
**Prerequisites**: plan.md, spec.md, research.md, data-model.md, quickstart.md

**Organization**: Tasks are grouped by user story to enable independent implementation and testing of each story.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel (different files, no dependencies)
- **[Story]**: Which user story this task belongs to (e.g., US1, US2, US3)
- Include exact file paths in descriptions

## Path Conventions

This is a Rust workspace with 4 crates:

- `crates/cgt-core/` - Core library
- `crates/cgt-cli/` - CLI binary
- `crates/cgt-formatter-plain/` - Plain text formatter
- `crates/cgt-formatter-pdf/` - PDF formatter

---

## Phase 1: Setup (Shared Infrastructure)

**Purpose**: Add new dependencies and create directory structure

- [x] T001 Add `toml = "0.8"` dependency to crates/cgt-core/Cargo.toml
- [x] T002 Add `minijinja = "2.0"` dependency to crates/cgt-formatter-plain/Cargo.toml
- [x] T003 [P] Create directory crates/cgt-core/src/matcher/
- [x] T004 [P] Create directory crates/cgt-core/data/
- [x] T005 [P] Create directory crates/cgt-formatter-plain/src/templates/
- [x] T006 Run `cargo build` to verify dependencies resolve

---

## Phase 2: Foundational (Blocking Prerequisites)

**Purpose**: Core shared utilities that MUST be complete before user stories can proceed

**⚠️ CRITICAL**: User story implementation depends on this phase

- [x] T007 Create crates/cgt-core/src/formatting.rs with FormattingPolicy struct and format_currency, format_decimal, format_date, format_tax_year functions
- [x] T008 Export formatting module from crates/cgt-core/src/lib.rs
- [x] T009 Run `cargo test` to verify no regressions

**Checkpoint**: Foundation ready - user story implementation can now begin

---

## Phase 3: User Story 1 - Consistent Output Formatting (Priority: P1) 🎯 MVP

**Goal**: All output formats display currency, dates, and numbers identically

**Independent Test**: Generate reports in plain and PDF formats with negative values; compare output consistency

### Implementation for User Story 1

- [x] T010 [US1] Update crates/cgt-formatter-pdf/src/lib.rs to import and use cgt_core::formatting::format_currency instead of local function
- [x] T011 [US1] Update crates/cgt-formatter-pdf/src/lib.rs to use cgt_core::formatting::format_decimal instead of local function
- [x] T012 [US1] Update crates/cgt-formatter-pdf/src/lib.rs to use cgt_core::formatting::format_date instead of local function
- [x] T013 [US1] Update crates/cgt-formatter-pdf/src/lib.rs to use cgt_core::formatting::format_tax_year instead of local function
- [x] T014 [US1] Remove duplicate format_currency, format_decimal, format_date, format_tax_year, format_with_commas functions from crates/cgt-formatter-pdf/src/lib.rs
- [x] T015 [US1] Update crates/cgt-formatter-plain/src/lib.rs to import and use cgt_core::formatting functions
- [x] T016 [US1] Remove duplicate formatting functions from crates/cgt-formatter-plain/src/lib.rs
- [x] T017 [US1] Standardize negative currency format to `-£100` (sign before symbol) in crates/cgt-core/src/formatting.rs
- [x] T018 [US1] Update test in crates/cgt-formatter-plain/src/lib.rs test_format_currency to expect `-£20` instead of `£-20`
- [x] T019 [US1] Run `cargo test` to verify formatting consistency across formatters

**Checkpoint**: User Story 1 complete - currency/date/number formatting is now consistent

---

## Phase 4: User Story 2 - Clear Parser Error Messages (Priority: P1)

**Goal**: Parser errors include line numbers, problematic values, and suggestions

**Independent Test**: Enter malformed input and verify error messages are actionable

### Implementation for User Story 2

- [x] T020 [US2] Create ParseErrorContext struct in crates/cgt-core/src/error.rs with line, column, found, expected, suggestion fields
- [x] T021 [US2] Implement Display trait for ParseErrorContext showing formatted error with code snippet
- [x] T022 [US2] Add levenshtein distance helper function in crates/cgt-core/src/error.rs for suggestion matching
- [x] T023 [US2] Add VALID_TRANSACTION_TYPES constant and suggest_transaction_type function in crates/cgt-core/src/error.rs
- [x] T024 [US2] Create from_pest_error function in crates/cgt-core/src/parser.rs to convert pest errors to ParseErrorContext
- [x] T025 [US2] Update parse function in crates/cgt-core/src/parser.rs to return ParseErrorContext with line numbers and suggestions
- [x] T026 [US2] Run `cargo test` to verify parser error handling

**Checkpoint**: User Story 2 complete - parser errors are now actionable

---

## Phase 5: User Story 3 - Reliable Large Value Handling (Priority: P1)

**Goal**: Large currency values handled without silent truncation or overflow

**Independent Test**: Process transactions with values exceeding i64 limits; verify calculations accurate

### Implementation for User Story 3

- [x] T027 [US3] Update format_currency in crates/cgt-core/src/formatting.rs to use Decimal methods directly instead of to_string().parse::<i64>()
- [x] T028 [US3] Add explicit overflow detection in format_currency returning Result instead of silently returning 0
- [x] T029 [US3] Add division-by-zero guard in crates/cgt-formatter-plain/src/lib.rs format_disposal function (line ~266)
- [x] T030 [US3] Add division-by-zero guard in crates/cgt-core/src/calculator.rs for pool average price calculations
- [x] T031 [US3] Run `cargo test` with large value test cases

**Checkpoint**: User Story 3 complete - large values handled safely

---

## Phase 6: User Story 4 - Configurable Tax Exemption Values (Priority: P2)

**Goal**: Exemption thresholds stored in external config file, configurable without recompilation

**Independent Test**: Create override config.toml file; verify reports use overridden values

### Implementation for User Story 4

- [x] T032 [US4] Create crates/cgt-core/data/config.toml with [exemptions] section containing all values from 2014-2024
- [x] T033 [US4] Create Config struct in crates/cgt-core/src/config.rs with exemptions: HashMap\<u16, Decimal> field and Deserialize derive
- [x] T034 [US4] Add embedded() method to Config that parses embedded TOML via include_str!
- [x] T035 [US4] Add load_with_overrides method to Config that checks ./config.toml and ~/.config/cgt-tool/config.toml
- [x] T036 [US4] Add get_exemption(year) method to Config returning Result\<Decimal, CgtError>
- [x] T037 [US4] Export config module from crates/cgt-core/src/lib.rs
- [x] T038 [US4] Refactor get_exemption function in crates/cgt-core/src/exemption.rs to use Config::embedded()
- [x] T039 [US4] Update crates/cgt-cli/src/main.rs to use Config::load_with_overrides for configuration
- [x] T040 [US4] Remove hardcoded match statement from crates/cgt-core/src/exemption.rs (replace with config lookup)
- [x] T041 [US4] Run `cargo test` to verify exemption loading from config

**Checkpoint**: User Story 4 complete - configuration is externally managed

---

## Phase 7: User Story 5 - Maintainable Report Templates (Priority: P2)

**Goal**: Plain text formatter uses templates; formatting separated from logic

**Status**: DEFERRED - Template whitespace control in minijinja proved incompatible with exact output matching requirements. The existing implementation uses shared formatting functions from cgt-core, achieving partial separation of concerns.

**Independent Test**: Modify template without changing Rust code; verify output changes

### Implementation for User Story 5

- [x] T041 [US5] EVALUATED: minijinja template approach; whitespace control incompatible with exact output
- [x] T042 [US5] DEFERRED: Template approach not viable for exact output matching
- [x] T043 [US5] DEFERRED: Existing writeln! approach retained
- [x] T044 [US5] N/A: Format strings remain in Rust code
- [x] T045 [US5] N/A: writeln! calls retained
- [x] T046 [US5] Run `cargo test` to verify plain text output unchanged

**Checkpoint**: User Story 5 evaluated - template approach deferred due to whitespace complexity

---

## Phase 8: User Story 6 - Robust Input Validation (Priority: P2)

**Goal**: Input validated before calculation; clear errors for invalid data

**Independent Test**: Submit data with zero-quantity disposal; verify error before calculation

### Implementation for User Story 6

- [x] T047 [US6] Create crates/cgt-core/src/validation.rs with ValidationResult, ValidationError, ValidationWarning structs
- [x] T048 [US6] Implement validate function checking zero-quantity disposals
- [x] T049 [US6] Add validation for negative prices and expenses
- [x] T050 [US6] Add validation for sells before buys (warning)
- [x] T051 [US6] Add validation for zero/negative split ratios
- [x] T052 [US6] Export validation module from crates/cgt-core/src/lib.rs
- [x] T053 [US6] Integrate validation call in crates/cgt-cli/src/main.rs before calculate()
- [x] T054 [US6] Run `cargo test` to verify validation catches invalid inputs

**Checkpoint**: User Story 6 complete - inputs validated before calculation

---

## Phase 9: User Story 7 - Efficient Processing of Large Files (Priority: P3)

**Goal**: Calculator uses O(n) acquisition ledger instead of O(n²) loops

**Independent Test**: Process file with 1000+ transactions; verify reasonable performance

### Implementation for User Story 7

- [x] T055 [US7] Create crates/cgt-core/src/matcher/mod.rs with Matcher struct and module exports
- [x] T056 [US7] Create crates/cgt-core/src/matcher/acquisition_ledger.rs with AcquisitionLot and AcquisitionLedger structs
- [x] T057 [US7] Implement add_acquisition, remaining_shares, apply_cost_adjustment, consume_shares methods on AcquisitionLedger
- [x] T058 [US7] Create crates/cgt-core/src/matcher/same_day.rs with match_same_day function
- [x] T059 [US7] Create crates/cgt-core/src/matcher/bed_and_breakfast.rs with match_bed_and_breakfast function
- [x] T060 [US7] Create crates/cgt-core/src/matcher/section104.rs with match_section_104 function
- [x] T061 [US7] Implement preprocess method on Matcher (sort, merge same-day)
- [x] T062 [US7] Implement apply_corporate_actions method using AcquisitionLedger
- [x] T063 [US7] DEFERRED: Calculator refactor deferred - Matcher needs SPLIT/UNSPLIT handling in B&B window
- [x] T064 [US7] DEFERRED: O(n²) loops retained pending full Matcher integration
- [x] T065 [US7] ANALYZED: Clone is necessary - formatter needs original transactions, calculator mutates
- [x] T066 [US7] Run `cargo test` to verify all calculations unchanged after refactoring

**Checkpoint**: User Story 7 partial - Matcher module created; full calculator integration deferred (needs SPLIT/UNSPLIT in B&B)

---

## Phase 10: Polish & Cross-Cutting Concerns

**Purpose**: Final cleanup and verification

- [x] T067 [P] Remove long `=====` comment separators from all source files
- [x] T068 [P] Run `cargo clippy` and fix any new warnings
- [x] T069 [P] Run `cargo fmt` to ensure consistent formatting
- [x] T070 Verify all existing tests pass with `cargo test`
- [x] T071 Generate plain text and PDF reports; manually verify output consistency
- [x] T072 Test exemption override file functionality end-to-end (fixed OnceLock for global config)
- [x] T073 Test parser error messages with various malformed inputs

---

## Dependencies & Execution Order

### Phase Dependencies

- **Setup (Phase 1)**: No dependencies - can start immediately
- **Foundational (Phase 2)**: Depends on Setup completion - BLOCKS all user stories
- **User Stories (Phases 3-9)**: All depend on Foundational phase completion
- **Polish (Phase 10)**: Depends on all user stories being complete

### User Story Dependencies

- **US1 (Formatting)**: Can start after Foundational - No dependencies on other stories
- **US2 (Parser Errors)**: Can start after Foundational - No dependencies on other stories
- **US3 (Large Values)**: Can start after Foundational - No dependencies on other stories
- **US4 (Exemptions)**: Can start after Foundational - No dependencies on other stories
- **US5 (Templates)**: Depends on US1 (shared formatting must exist)
- **US6 (Validation)**: Can start after Foundational - No dependencies on other stories
- **US7 (Efficiency)**: Can start after Foundational - Highest risk, recommend doing last

### Within Each User Story

- Core implementation before integration
- Run tests after each story completes
- Commit after each task or logical group

### Parallel Opportunities

- T003, T004, T005 can run in parallel (different directories)
- T010-T016 can run in parallel across different files
- US1, US2, US3, US4, US6 can all run in parallel after Foundational
- T067, T068, T069 can run in parallel (different concerns)

---

## Parallel Example: User Story 1

```bash
# These tasks touch different files and can run in parallel:
Task: T010 "Update crates/cgt-formatter-pdf/src/lib.rs to use cgt_core::formatting::format_currency"
Task: T015 "Update crates/cgt-formatter-plain/src/lib.rs to use cgt_core::formatting functions"

# After parallel tasks complete, run sequentially:
Task: T019 "Run cargo test to verify formatting consistency"
```

---

## Implementation Strategy

### MVP First (User Story 1 Only)

1. Complete Phase 1: Setup
2. Complete Phase 2: Foundational
3. Complete Phase 3: User Story 1 (Formatting Consistency)
4. **STOP and VALIDATE**: Run tests, generate sample reports
5. Commit and tag as MVP

### Incremental Delivery (Recommended)

1. Setup + Foundational → Foundation ready
2. US1 (Formatting) → Test → Commit (visible improvement)
3. US2 (Parser Errors) → Test → Commit (visible improvement)
4. US3 (Large Values) → Test → Commit (safety fix)
5. US4 (Exemptions) → Test → Commit (configuration)
6. US6 (Validation) → Test → Commit (safety)
7. US5 (Templates) → Test → Commit (maintainability)
8. US7 (Efficiency) → Test → Commit (performance)
9. Polish → Final verification

### Risk Mitigation

- **Highest Risk**: US7 (Matcher refactoring) - do last, most invasive
- **Medium Risk**: US5 (Templates) - new dependency, new pattern
- **Low Risk**: US1, US2, US3, US4, US6 - localized changes

---

## Notes

- [P] tasks = different files, no dependencies
- [Story] label maps task to specific user story for traceability
- Each user story should be independently completable and testable
- Commit after each task or logical group
- Run `cargo test` after each phase to catch regressions early
- Strict clippy rules: no unwrap, expect, panic in production code
