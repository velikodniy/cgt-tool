# Design: Code Quality Improvements

## Context

This refactoring addresses findings from a code review. The changes span multiple crates but are independent improvements that don't introduce new features or change external behavior.

## Goals / Non-Goals

### Goals

- Improve robustness against edge cases (division by zero, case sensitivity)
- Improve maintainability (smaller files, consistent patterns)
- Improve testability (explicit config passing)
- Remove dead code (unused error variants)

### Non-Goals

- Adding new features
- Changing public API signatures (except adding optional parameters)
- Changing calculation results

## Decisions

### Division Safety Approach

**Decision**: Add early-return guards rather than changing the calculation logic.

**Rationale**: If `total_sell_amount` is zero, there's nothing to match anyway. An early return is clearer than conditional division.

```rust
// Before
let proportion = matched_qty / total_sell_amount;

// After
if total_sell_amount == Decimal::ZERO {
    return Ok(None);
}
let proportion = matched_qty / total_sell_amount;
```

**Alternative considered**: Using `checked_div()` - rejected because it adds complexity and the early-return case is semantically correct (zero sell = no matching needed).

### Symbol Case Normalization

**Decision**: Uppercase symbols at lookup time in `AwardsData::get_fmv()`.

**Rationale**: The rest of the codebase uppercases symbols (e.g., `Transaction` deserialization). The awards file should follow the same convention.

```rust
// Before
self.fmv_map.get(&(symbol.to_string(), *date))

// After
self.fmv_map.get(&(symbol.to_uppercase(), *date))
```

**Note**: Also uppercase symbols when inserting into the map during parsing.

### CSV Extra Column Tolerance

**Decision**: Tolerate extra columns with empty values in Schwab CSV exports.

**Rationale**: Schwab CSV exports can vary over time and may include additional columns. The parser should gracefully ignore columns it doesn't recognize rather than failing.

**Current implementation**: Already handled via `flexible(true)` on the CSV reader, which allows variable field counts per row.

**Action**: Add explicit test case and documentation comment to make this intentional tolerance clear.

### Config Flexibility

**Decision**: Add explicit config parameter function alongside global accessor.

**Rationale**: Keeps backward compatibility while enabling testability.

```rust
// New function for explicit config
pub fn get_exemption_with_config(year: u16, config: &Config) -> Result<Decimal, CgtError> {
    config.get_exemption(year)
}

// Existing function unchanged, delegates to new one
pub fn get_exemption(year: u16) -> Result<Decimal, CgtError> {
    get_exemption_with_config(year, get_config())
}
```

### Test Consolidation Strategy

**Decision**: Use a single test with inline data rather than 11 separate test functions.

**Rationale**: The current tests duplicate structure and just verify data. A single parameterized test is clearer.

```rust
#[test]
fn test_exemption_known_years() {
    let cases = [
        (2014, 11000),
        (2015, 11100),
        (2016, 11100),
        // ... etc
    ];

    for (year, expected) in cases {
        assert_eq!(
            get_exemption(year).expect(&format!("year {}", year)),
            Decimal::from(expected),
            "Year {} should have exemption {}",
            year,
            expected
        );
    }
}
```

### String Extraction from Server

**Decision**: Move all string constants to `resources.rs`, keep them as `const` or `static` for zero-cost inclusion.

**Rationale**: Reduces cognitive load when reading server.rs. Resources module is the natural home for text content.

**Scope**:

- `HINT_*` constants (7 items)
- `DSL_SYNTAX_REFERENCE`
- `EXAMPLE_TRANSACTION`

### Unused Error Variant Removal

**Decision**: Simply remove the variants. No migration needed since they're never constructed.

**Verification**: `rg "InvalidParameter|ResourceNotFound|DisposalNotFound" crates/cgt-mcp/src/` shows no construction sites.

### let-else Standardization

**Decision**: Convert `match ... { Some(x) => x, None => return }` patterns to `let Some(x) = ... else { return }`.

**Rationale**: `let-else` is more concise and is already used elsewhere in the codebase (e.g., `bed_and_breakfast.rs:27`).

## Risks / Trade-offs

| Risk                                              | Mitigation                                                         |
| ------------------------------------------------- | ------------------------------------------------------------------ |
| Division guard might hide legitimate bugs         | Add test case explicitly testing zero sell amount returns None     |
| Case normalization might break existing workflows | Already done in Transaction deserialization; awards is the outlier |
| Config changes might affect concurrent access     | OnceLock remains for global case; explicit param is new path       |

## Migration Plan

No migration needed - these are internal improvements with no public API changes.

## Open Questions

None - all decisions are straightforward refactoring with clear rationale.
