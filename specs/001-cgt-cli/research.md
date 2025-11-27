# Research: Capital Gains Tax (CGT) CLI Tool

**Feature**: `001-cgt-cli`
**Date**: 2025-11-27

## Decisions & Rationale

### 1. PEG Parser Library: `pest` vs `nom` vs `chumsky`

- **Decision**: **`pest`**
- **Rationale**:
  - User explicitly requested a PEG parser.
  - `pest` uses a separate `.pest` grammar file, which makes the DSL definition declarative and readable ("Deep Modules").
  - Excellent error reporting (Constitution Principle IV: UX).
  - WASM compatible.
- **Alternatives Considered**:
  - `nom`: Faster, but is a parser combinator library, not strict PEG. Code can get verbose.
  - `chumsky`: Great error recovery, but combinator-based.
  - `lalrpop`: LR(1) parser, not PEG.

### 2. Financial Mathematics: `rust_decimal`

- **Decision**: **`rust_decimal`**
- **Rationale**:
  - **Constitution Principle II (Safety)**: Floating point math (f64) is non-negotiable for financial calculations due to rounding errors.
  - `rust_decimal` provides 128-bit fixed-point precision, suitable for currency and tax calculations.
  - Supports serialization via `serde`.
- **Alternatives Considered**:
  - `f64`: Rejected due to precision issues (0.1 + 0.2 != 0.3).
  - `bigdecimal`: Arbitrary precision, but slower and likely overkill for standard CGT. `rust_decimal` fits in stack/registers better.

### 3. Date/Time Library: `chrono`

- **Decision**: **`chrono`**
- **Rationale**:
  - The standard for Rust time.
  - `NaiveDate` is perfect for transaction dates (no time/timezone needed for daily matching rules).
  - `serde` support features.
  - WASM compatible (via `wasm-bindgen` if needed later, or just pure logic).

### 4. CLI Framework: `clap`

- **Decision**: **`clap` (Derive API)**
- **Rationale**:
  - Industry standard for Rust CLIs.
  - "Derive" pattern keeps code declarative and simple (Constitution Principle I).
  - Generates help messages automatically (Constitution Principle IV).

### 5. Workspace Structure

- **Decision**: **Cargo Workspace with `cgt-core` and `cgt-cli`**
- **Rationale**:
  - **WASM Requirement**: `cgt-core` must not depend on system I/O or CLI crates to be easily compiled to WASM later.
  - **Testing**: Allows unit testing core logic independently of CLI argument parsing.

### 6. JSON Serialization

- **Decision**: **`serde` + `serde_json`**
- **Rationale**:
  - Rust ecosystem standard.
  - Required for the "JSON output" and "Schema" requirements.
  - `schemars` will be used to generate the JSON Schema automatically from structs (Constitution Principle II: Automation).

## Unknowns Resolved

- **WASM Friendliness**: `pest`, `rust_decimal`, `chrono`, and `serde` all support WASM targets.
- **PEG**: `pest` fulfills this specific requirement perfectly.

## HMRC CGT Rule Clarification (Implemented Logic)

**Research Limitations**: Direct access to current, authoritative HMRC guidance (e.g., specific PDF documents on gov.uk) was not possible due to tool limitations (404 on known links, web search returning agent clarification requests). The following rules are documented as implemented in `calculator.rs` and are based on general understanding of UK CGT shares matching rules and the behavior of the `cgtcalc` reference tool.

### Matching Rules Order (HMRC HS284 Equivalent)

The calculator strictly follows the HMRC prescribed order for matching disposals with acquisitions:

1. **Same Day Rule**: Disposals are first matched against acquisitions of the same share of the same class made on the same day.
2. **Bed & Breakfasting Rule**: Remaining disposals are matched against acquisitions of the same share of the same class made in the 30 days following the disposal.
3. **Section 104 Pooling**: Any remaining disposals are matched against the Section 104 holding (the share pool).

### Specific Rule Interpretations / Assumptions

- **Same Day Merging**: For a given asset (ticker) on a given date, all `BUY` transactions are implicitly merged into a single `BUY` event (weighted average cost/expenses) and all `SELL` transactions are merged into a single `SELL` event. This is done *before* matching rules are applied to simplify processing, consistent with how HMRC aggregates same-day trades.
- **Decimal Precision**: All calculations use `rust_decimal` for exact 128-bit decimal arithmetic, avoiding floating-point inaccuracies.
- **Bed & Breakfasting with Corporate Events**: The `calculator.rs` code includes logic to adjust quantities for splits (`Split`) and reverse splits (`Unsplit`) that occur between the disposal date and the subsequent acquisition date within the 30-day B&B window. This ensures fair matching by normalizing quantities to the same "basis" (pre-event quantity).
- **Section 104 Pool**:
  - All acquisitions not matched by Same Day or B&B rules contribute to a single Section 104 pool for that asset.
  - The pool tracks `quantity` and `total_cost`.
  - Disposals from the pool use the average cost (`total_cost / quantity`) of the pool.
  - **Capital Returns (`CAPRETURN`)**: These are treated as "Small Capital Distributions" that reduce the `total_cost` of the pool. If the cost pool goes negative, the excess is not currently recorded as an immediate gain but rather handled by subsequent disposals (effectively leading to a higher gain later).
  - **Splits (`Split`/`Unsplit`)**: These only affect the `quantity` of the pool (multiplying or dividing by the ratio) while the `total_cost` remains unchanged.
- **Dividends (`DIVIDEND`)**: These are currently processed but do not affect Capital Gains Tax calculations (assumed to be income tax related). They are effectively ignored by the CGT calculation logic.
- **Error Handling**: Selling more shares than held (and not matched by Same Day/B&B) results in an `InvalidTransaction` error (Negative Holding).
- **Tax Year**: Reports are generated for a single specified tax year (starting April 6th and ending April 5th of the following year). The input `year` argument corresponds to the start year (e.g., `2018` for `2018/2019`).
- **Test Data Discrepancies**: Discrepancies between `cgtcalc`'s text output and our calculator's precise decimal results (especially for `Total Gain/Loss`) are noted in test assertions using an `epsilon` of `1.0` to account for `cgtcalc`'s apparent rounding behavior in its summary/match output. Our calculator prioritizes exact arithmetic.

## Required Actions / Further Verification

- **External HMRC Document Review**: A manual review of authoritative HMRC guidance (e.g., the latest version of HS284 or equivalent) is required to formally verify these interpretations against current tax law.
- **Edge Cases for Capital Returns**: Further clarification is needed on how HMRC treats capital returns that cause the cost pool to go negative (i.e., immediate gain vs. deferring gain). This could impact `TaxReport.total_gain`.
- **Test Data Validation**: While existing test cases cover core rules, any new HMRC guidance or specific scenarios identified during external review should be translated into new test cases.
