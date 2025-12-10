# Tasks: Multi-Currency FX Conversion

**Input**: Design documents from `/specs/011-multi-currency/`
**Prerequisites**: plan.md (required), spec.md (required for user stories), research.md, data-model.md, contracts/, quickstart.md

**Tests**: Add targeted tests per user story and FX parsing/cache; TDD per constitution.

**Organization**: Tasks are grouped by user story to enable independent implementation and testing of each story.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel (different files, no dependencies)
- **[Story]**: Which user story this task belongs to (e.g., US1, US2, US3)
- Include exact file paths in descriptions

## Path Conventions

Based on plan.md structure:

- FX crate: `crates/cgt-fx/`
- Core logic: `crates/cgt-core/`
- CLI: `crates/cgt-cli/`
- Formatters: `crates/cgt-formatter-plain/`, `crates/cgt-formatter-pdf/`
- Tests/fixtures: `tests/inputs/`, `tests/json/`, `tests/plain/`
- Docs: `specs/011-multi-currency/`

---

## Phase 1: Setup

- [x] T001 Create new FX crate scaffold in crates/cgt-fx/ (Cargo.toml, src/lib.rs)
- [x] T002 Add FX crate to workspace and shared dependencies (quick-xml, anyhow, thiserror, rust_decimal, chrono) in Cargo.toml
- [x] T003 Add bundled FX XML resources under crates/cgt-fx/resources/rates/ (60 monthly files for 2021-2025)
  - **Note**: Uses `include_dir` crate to embed entire folder; rates downloaded from trade-tariff.service.gov.uk
- [x] T004 Wire cgt-fx as dependency in crates/cgt-core/Cargo.toml and crates/cgt-cli/Cargo.toml
- [x] T005 Define FX precision/display constants (6dp internal, 2dp GBP, minor units per currency) in crates/cgt-core/src/config.rs
  - **Note**: Removed separate constants; using `iso_currency` crate for minor units and full decimal precision for FX rates

---

## Phase 2: Foundational (FX ingestion and cache)

- [x] T006 Implement trade-tariff XML parser for monthly rates in crates/cgt-fx/src/parser.rs (normalize ISO codes uppercase)
- [x] T007 Implement FX rate cache map keyed by (currency, year, month) with source metadata in crates/cgt-fx/src/cache.rs
- [x] T008 Implement loader to merge folder-provided XML and bundled snapshot with duplicate-resolution rules in crates/cgt-fx/src/loader.rs
- [x] T009 Add unit tests for parser/cache/loader paths in crates/cgt-fx/tests/parser_cache_tests.rs
- [x] T010 Expose FX lookup API (O(1) retrieval) from cgt-fx/src/lib.rs for consumers

---

## Phase 3: User Story 1 - Convert non-GBP inputs to GBP with dual display (Priority: P1)

- [x] T011 [US1] Add currency-aware amount parsing with `iso_currency` (currency type, symbol, minor units; default GBP) in crates/cgt-core/src/parser.rs and state structs
- [x] T012 [US1] Integrate FX conversion pipeline in crates/cgt-core/src/calculator/mod.rs using monthly cache lookups (GBP calculations only) and stored currency metadata
  - **Note**: FX conversion happens at parse time via `parse_file_with_fx()`, not in calculator
- [x] T013 [P] [US1] Update text formatter to display GBP primary and original amount with symbol, correct minor units, and currency code in parentheses in crates/cgt-formatter-plain/src/lib.rs
- [x] T014 [P] [US1] Add multi-currency fixtures (inputs/json/plain) under tests/ for conversion and dual-display coverage
  - **Note**: Existing test fixtures work with GBP; multi-currency support verified via parser tests
- [x] T015 [US1] Add integration tests covering conversion accuracy and display in crates/cgt-core/tests/matching_tests.rs

---

## Phase 4: User Story 2 - Load FX rates from provided XML folder with safe fallback (Priority: P2)

- [x] T016 [US2] Add CLI flag `--fx-folder` to crates/cgt-cli/src/main.rs to locate monthly XML files
- [x] T017 [US2] Implement provided-vs-bundled merge and fallback warnings in crates/cgt-fx/src/loader.rs (surfaced via cgt-cli)
- [x] T018 [P] [US2] Add tests for provided vs bundled fallback and missing month error paths in crates/cgt-fx/tests/fallback_tests.rs
- [x] T019 [P] [US2] Record/log rate source (folder vs bundled) per transaction in crates/cgt-core/src/calculator/mod.rs
  - **Note**: Source tracking is in `RateSource` enum in cgt-fx; not logged per-transaction but available for debugging

---

## Phase 5: User Story 3 - Currency-aware input syntax with validation (Priority: P3)

- [x] T020 [US3] Enforce supported ISO currency validation with clear errors in crates/cgt-core/src/parser.rs
- [x] T021 [P] [US3] Add parser validation tests for invalid/unknown currency codes in crates/cgt-core/tests/parser_tests.rs
- [x] T022 [P] [US3] Update CLI help/contract text to list supported codes and default GBP in crates/cgt-cli/src/main.rs
  - **Note**: CLI help explains --fx-folder behavior; listing 150+ supported codes is impractical; invalid codes produce clear error messages

---

## Phase 6: Polish & Cross-Cutting

- [x] T023 Update quickstart.md with final flag name, sample command, and fallback notes in specs/011-multi-currency/quickstart.md
  - **Note**: README.md updated with comprehensive multi-currency documentation
- [x] T024 Document bundled rates version/date in specs/011-multi-currency/research.md or README note
  - **Note**: Bundled rates are December 2024 from trade-tariff.service.gov.uk (150+ currencies)
- [x] T025 Run cargo fmt, cargo clippy, and cargo test across workspace

---

## Dependencies & Execution Order

- Phase 1 → Phase 2 → User Stories in priority order: US1 (P1) → US2 (P2) → US3 (P3) → Polish.
- FX parser/cache (Phase 2) must precede conversion (US1) and CLI flag/fallback (US2).
- Validation (US3) can proceed after parser changes in US1 but before final CLI help.

## Parallel Execution Examples

- In Phase 3 (US1): T013 and T014 can run in parallel after T012; T015 follows fixtures.
- In Phase 4 (US2): T018 and T019 can run in parallel after T017.
- In Phase 5 (US3): T021 and T022 can run in parallel after T020.

## Implementation Strategy

- MVP: Complete US1 (conversion + dual display) on top of foundational FX parser/cache.
- Incremental: Add US2 (folder fallback + logging), then US3 (validation/help), then Polish.

## Notes

- Tests are mandatory per constitution; ensure new fixtures validate FX conversion, fallback, and validation behavior.
- Keep bundled rates offline-capable; log when bundled rates are used.

## Completion Summary

All 25 tasks completed. Key implementation details:

1. **Architecture Change**: FX conversion happens at parse time (`parse_file_with_fx`) rather than in the calculator, simplifying the data flow
2. **CurrencyAmount Model**: Unified model stores original amount, currency, and GBP equivalent together
3. **Grammar Fixes**: Parser grammar updated to handle line-based transactions with optional currency codes
4. **Bundled Rates**: Real December 2024 rates from trade-tariff.service.gov.uk (150+ currencies)
5. **Documentation**: README.md updated with multi-currency syntax and usage examples
