# Design: Add CSV Format Support for Schwab Equity Awards

## Context

Schwab equity awards exports come in two formats:

1. **JSON**: Simple array structure with Symbol, EventDate, FairMarketValuePrice
2. **CSV**: Complex paired-row format where each lapse event has:
   - Row 1: Transaction details (Date, Action, Symbol, Quantity, etc.)
   - Row 2: Award details (AwardDate, FairMarketValuePrice, etc.) with empty transaction fields

The CSV format is more complex because award details are on a separate row following the transaction row, creating a paired-row structure that must be parsed carefully.

**Constraints:**

- Must maintain backward compatibility with existing JSON format
- Must support WASM (no filesystem operations)
- Must preserve existing 7-day FMV lookback behavior
- Must use file extension for format detection (no explicit format flag)

## Goals / Non-Goals

**Goals:**

- Support both JSON and CSV awards formats seamlessly
- Auto-detect format based on file extension
- Parse Schwab CSV paired-row format correctly
- Extract FairMarketValuePrice and AwardDate from award rows
- Maintain same `AwardsData` internal representation

**Non-Goals:**

- Support other CSV formats (focus on Schwab's specific structure)
- Complex CSV validation beyond structural requirements
- Support for mixed formats (multiple awards files)

## Decisions

### Decision 1: File Extension-Based Format Detection

**Rationale:** Simple, user-friendly approach that avoids additional CLI flags. Users already know what format they exported from Schwab.

**Implementation:**

```rust
pub fn parse_awards(content: &str, filename: Option<&str>) -> Result<AwardsData, ConvertError> {
    if let Some(name) = filename {
        if name.ends_with(".csv") {
            return parse_awards_csv(content);
        }
    }
    // Default to JSON for backward compatibility
    parse_awards_json(content)
}
```

**Alternatives considered:**

- Content-based detection: Too fragile, could misidentify
- Explicit flags (--awards-json, --awards-csv): More complexity, worse UX

### Decision 2: Paired-Row CSV Parsing Strategy

**Rationale:** The Schwab CSV format has transaction details on one row and award details on the next. We need to:

1. Identify lapse transaction rows (Action = "Lapse")
2. Read the subsequent row for award details
3. Extract AwardDate and FairMarketValuePrice from the award row

**Structure:**

```csv
Date,Action,Symbol,...,AwardDate,FairMarketValuePrice,...
"MM/DD/YYYY","Lapse","TICKER",...,"","","",...  <- Transaction row
"","","","",...,"MM/DD/YYYY","$X.XX",...        <- Award row (paired)
```

**Implementation approach:**

- Use CSV reader with headers
- Track iteration state to handle paired rows
- Skip rows without AwardDate (transaction-only rows)
- Extract Symbol from transaction row, AwardDate/FMV from award row

**Alternatives considered:**

- Single-row parser: Doesn't match Schwab format
- Look-ahead buffering: More complex, unnecessary

### Decision 3: Unified `AwardsData` Representation

**Rationale:** Both JSON and CSV produce the same internal `AwardsData` structure (HashMap of (symbol, date) -> FMV). This keeps downstream code unchanged.

**No changes needed to:**

- `AwardsData::get_fmv()` method
- 7-day lookback logic
- Transaction processing in `mod.rs`

### Decision 4: Error Handling for CSV Format

**Rationale:** CSV parsing can fail in multiple ways (missing columns, invalid dates, malformed prices). Provide clear error messages.

**Error types to handle:**

- Missing required columns (AwardDate, FairMarketValuePrice, Symbol)
- Invalid date format (not MM/DD/YYYY)
- Invalid price format (after removing $, commas)
- CSV structural errors (via csv crate)

## Risks / Trade-offs

| Risk                         | Mitigation                                                       |
| ---------------------------- | ---------------------------------------------------------------- |
| Schwab CSV format changes    | Version detection not implemented; rely on clear error messages  |
| Paired-row parsing fragility | Comprehensive tests with edge cases (single rows, missing pairs) |
| Format misdetection          | Default to JSON for backward compatibility; clear error messages |
| Performance impact           | Minimal - single pass through CSV, same as JSON                  |

## Migration Plan

No migration needed - this is additive functionality. Existing JSON usage continues to work.

**Rollout:**

1. Implement CSV parser with paired-row handling
2. Add format detection wrapper
3. Update tests with CSV fixtures
4. Update documentation to mention CSV support

## Open Questions

None - CSV structure is well-defined from examining real exports.
