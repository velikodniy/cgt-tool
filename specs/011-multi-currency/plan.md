# Implementation Plan: Multi-Currency FX Conversion

**Branch**: `011-multi-currency` | **Date**: 2025-12-09 | **Spec**: specs/011-multi-currency/spec.md
**Input**: Feature specification from `/specs/011-multi-currency/spec.md`

## Summary

Enable multi-currency inputs by adding ISO 4217 currency support with monthly FX conversion to GBP, dual display (GBP primary, original in parentheses), a new CLI `--fx-folder` for monthly XML rates from trade-tariff, and a dedicated FX conversion crate with cached lookups and 6dp internal precision.

## Technical Context

**Language/Version**: Rust 2024 edition (stable workspace)
**Primary Dependencies**: rust_decimal, chrono, pest, serde, iso_currency; planned XML parsing via quick-xml; error handling via anyhow/thiserror; new FX conversion crate to be added to the workspace
**Storage**: None (in-memory rate cache)
**Testing**: cargo test with fixture-based coverage; TDD per constitution; clippy/rustfmt enforced
**Target Platform**: Cross-platform CLI/library (macOS/Linux/Windows)
**Project Type**: Rust workspace (CLI + core libs + formatter crates + new FX crate)
**Performance Goals**: Process ~10k transactions with mixed currencies in \<2 minutes end-to-end; O(1) FX lookups; avoid repeated XML reads
**Constraints**: Calculations in GBP only; 6dp internal FX precision; display GBP to 2dp and original currency to its minor units; display original currency symbol where available; fail fast on missing currency/month after bundled fallback; canonical currency type from `iso_currency` for code/symbol/minor units
**Scale/Scope**: Dozens of currencies, monthly rates; batch sizes in the tens of thousands transactions per run

## Constitution Check

- Principle III (Modern Testing Standards): Plan includes TDD and keeps existing tests untouched; new tests required for FX paths.
- Principle VI (Domain Mastery & Verification): Monthly rates selected by transaction month; explicit fallbacks and erroring on missing rates to avoid silent drift.
- Safety & Robustness: Fail fast on missing rates or malformed XML; no silent fallbacks.
- No violations expected; re-check after design to confirm no gate regressions.

## Project Structure

### Documentation (this feature)

```text
specs/011-multi-currency/
├── plan.md              # This file
├── research.md          # Decisions for FX handling, precision, sources
├── data-model.md        # Entities for currency amounts and FX cache
├── quickstart.md        # How to run with FX folder/bundled rates
├── contracts/           # CLI flag and XML format contract
│   └── fx-contracts.md
└── tasks.md             # Generated later by /speckit.tasks
```

### Source Code (repository root)

```text
crates/
├── cgt-core/            # Core CGT logic and DSL parser
├── cgt-cli/             # CLI entrypoint (to add --fx-folder)
├── cgt-formatter-plain/ # Text formatter
├── cgt-formatter-pdf/   # PDF formatter
└── cgt-fx/              # NEW: FX conversion crate (XML ingest, cache, lookup)

tests/
├── inputs/              # CGT input fixtures (may add currency codes)
├── json/                # Expected JSON outputs
└── plain/               # Expected text outputs
```

**Structure Decision**: Use existing Cargo workspace; add a new `crates/cgt-fx` crate for FX ingestion/cache; wire into `cgt-core`/`cgt-cli`; reuse existing `tests/` fixture layout for new currency cases.

## Complexity Tracking

None.
