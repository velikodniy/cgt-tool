# Quickstart: Codebase Quality Refactoring

**Feature**: 009-codebase-refactoring
**Date**: 2025-12-09

## Overview

This refactoring improves code quality across the cgt-tool codebase without changing external behavior. Key changes:

1. **Shared formatting** - Single source of truth for currency/date formatting
2. **Matcher extraction** - Calculator split into isolated matching passes
3. **Acquisition ledger** - O(n) corporate action processing replacing O(n²) loops
4. **Validation pass** - Pre-calculation input validation with clear errors
5. **Template-based plain text** - Plain formatter uses templates like PDF
6. **Externalized exemptions** - TOML config with embedded defaults

## Prerequisites

- Rust 2024 edition
- Existing cgt-tool workspace builds successfully
- All existing tests pass

## New Dependencies

Add to `crates/cgt-core/Cargo.toml`:

```toml
toml = "0.8"
dirs = "5.0"  # Optional, for XDG config paths
```

Add to `crates/cgt-formatter-plain/Cargo.toml`:

```toml
minijinja = "2.0"
```

## Implementation Order

### Phase 1: Foundation (No behavior changes)

1. **Create `cgt-core/src/formatting.rs`**

   - Move `format_currency`, `format_decimal`, `format_date`, `format_tax_year` from formatters
   - Implement `FormattingPolicy` struct
   - Standardize negative currency as `-£100` (sign before symbol)

2. **Update formatters to use shared formatting**

   - Replace local functions with `cgt_core::formatting::*`
   - Run tests to verify output unchanged

3. **Create `cgt-core/data/exemptions.toml`**

   - Extract hardcoded values from `exemption.rs`
   - Embed with `include_str!`

4. **Refactor `exemption.rs`**

   - Parse embedded TOML
   - Add override file support
   - Maintain `get_exemption()` API for compatibility

### Phase 2: Validation

5. **Create `cgt-core/src/validation.rs`**

   - Implement `validate()` function
   - Return `ValidationResult` with errors/warnings

6. **Integrate validation into CLI**

   - Call `validate()` before `calculate()`
   - Display warnings, abort on errors

### Phase 3: Matcher Extraction

7. **Create `cgt-core/src/matcher/` module**

   - Start with `mod.rs` re-exporting from `calculator.rs`
   - Extract `AcquisitionLedger` first (lowest risk)

8. **Extract matching passes one at a time**

   - `same_day.rs` - Same-day rule
   - `bed_and_breakfast.rs` - B&B rule
   - `section104.rs` - Section 104 pool

9. **Refactor `calculator.rs` to use Matcher**

   - Replace inline logic with `Matcher` calls
   - Verify all tests pass after each extraction

### Phase 4: Plain Text Templates

10. **Add minijinja templates**

    - Create `cgt-formatter-plain/src/templates/report.txt.tera`
    - Port format logic to template

11. **Refactor plain formatter**

    - Build data dict (like PDF formatter)
    - Render template
    - Verify output identical

### Phase 5: Parser Errors

12. **Create `ParseErrorContext` in `error.rs`**

    - Rich error with line/column/suggestions

13. **Enhance parser error handling**

    - Wrap pest errors with context
    - Add transaction type suggestions

## Testing Strategy

1. **Run full test suite after each file change**

   ```bash
   cargo test
   ```

2. **Verify output consistency**

   - Compare plain text output before/after formatting changes
   - Compare PDF output before/after formatting changes

3. **Add new unit tests for**

   - Each Matcher pass in isolation
   - AcquisitionLedger operations
   - Validation cases
   - ParseErrorContext formatting

## Key Files Modified

| File                                                | Change Type                          |
| --------------------------------------------------- | ------------------------------------ |
| `cgt-core/src/lib.rs`                               | Add exports for new modules          |
| `cgt-core/src/formatting.rs`                        | NEW: Shared formatting               |
| `cgt-core/src/validation.rs`                        | NEW: Input validation                |
| `cgt-core/src/matcher/mod.rs`                       | NEW: Matcher orchestration           |
| `cgt-core/src/matcher/acquisition_ledger.rs`        | NEW: FIFO ledger                     |
| `cgt-core/src/matcher/same_day.rs`                  | NEW: Same-day matching               |
| `cgt-core/src/matcher/bed_and_breakfast.rs`         | NEW: B&B matching                    |
| `cgt-core/src/matcher/section104.rs`                | NEW: Section 104 matching            |
| `cgt-core/src/exemption.rs`                         | Refactored: TOML-based               |
| `cgt-core/src/calculator.rs`                        | Refactored: Uses Matcher             |
| `cgt-core/src/error.rs`                             | Enhanced: ParseErrorContext          |
| `cgt-core/data/exemptions.toml`                     | NEW: Embedded config                 |
| `cgt-formatter-plain/src/lib.rs`                    | Refactored: Template-based           |
| `cgt-formatter-plain/src/templates/report.txt.tera` | NEW: Plain template                  |
| `cgt-formatter-pdf/src/lib.rs`                      | Refactored: Shared formatting        |
| `cgt-cli/src/main.rs`                               | Updated: Validation, reduced cloning |

## Verification Checklist

- [ ] All existing tests pass
- [ ] `cargo clippy` passes (no unwrap, expect, panic)
- [ ] Plain text output identical before/after
- [ ] PDF output identical before/after
- [ ] New tests cover Matcher passes
- [ ] New tests cover validation cases
- [ ] Exemption override file works
- [ ] Parser errors show line numbers and suggestions

## Rollback Plan

Each phase is independent. If issues arise:

1. Revert the problematic commit
2. Previous phases remain working
3. Re-attempt with adjusted approach

The Matcher extraction (Phase 3) is highest risk. If needed, can ship Phases 1-2 and defer Matcher to a follow-up.
