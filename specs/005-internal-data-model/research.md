# Research: Internal Data Model Improvements

**Feature**: 005-internal-data-model
**Date**: 2025-12-08

## Research Topics

### 1. TaxPeriod Type Design

**Decision**: Implement `TaxPeriod` as a newtype wrapper around a single `u16` (start year), with custom serialization to "YYYY/YY" format.

**Rationale**:

- Storing only the start year (e.g., 2023) is sufficient since UK tax years always span consecutive years
- Single field is simpler than storing both years
- Custom `Serialize`/`Deserialize` implementations output "2023/24" format
- Validation rejects years outside reasonable range (1900-2100)
- `From<NaiveDate>` derives the tax year from any date

**Alternatives Considered**:

| Alternative                   | Rejected Because                                    |
| ----------------------------- | --------------------------------------------------- |
| `String` field                | No validation, allows invalid values like "2023/27" |
| Two `u16` fields (start, end) | Redundant; end is always start+1                    |
| Enum with predefined years    | Not extensible, verbose                             |

**Implementation Pattern**:

```rust
pub struct TaxPeriod(u16);

impl TaxPeriod {
    pub fn new(start_year: u16) -> Result<Self, CgtError> {
        if start_year < 1900 || start_year > 2100 {
            return Err(CgtError::InvalidTaxYear(start_year));
        }
        Ok(Self(start_year))
    }

    pub fn from_date(date: NaiveDate) -> Self {
        let year = date.year() as u16;
        let month = date.month();
        // Tax year starts April 6
        if month < 4 || (month == 4 && date.day() < 6) {
            Self(year - 1)
        } else {
            Self(year)
        }
    }
}

// Serializes to "2023/24"
impl Serialize for TaxPeriod { ... }
// Deserializes from "2023/24"
impl Deserialize for TaxPeriod { ... }
```

### 2. Disposal vs Match Hierarchy

**Decision**: Introduce a `Disposal` struct that contains the sale details and a `Vec<Match>` for how it was matched.

**Rationale**:

- A single sale can be matched by multiple rules (same-day, B&B, Section 104)
- Preserves the relationship between sale event and its matching breakdown
- Enables report formatters to show "Sale of 100 shares → 50 same-day, 50 S104"
- Aligns with HMRC's view of disposals as the taxable events

**Alternatives Considered**:

| Alternative                          | Rejected Because                       |
| ------------------------------------ | -------------------------------------- |
| Keep flat matches, add `disposal_id` | Requires manual grouping in formatters |
| Duplicate sale info in each match    | Data redundancy, inconsistency risk    |

**Implementation Pattern**:

```rust
pub struct Disposal {
    pub date: NaiveDate,
    pub ticker: String,
    pub quantity: Decimal,
    pub proceeds: Decimal,
    pub matches: Vec<Match>,
}

pub struct Match {
    pub rule: MatchRule,
    pub quantity: Decimal,
    pub allowable_cost: Decimal,
    pub gain_or_loss: Decimal,
    pub acquisition_date: Option<NaiveDate>, // Present for B&B
}
```

### 3. B&B Acquisition Date Tracking

**Decision**: Add `acquisition_date: Option<NaiveDate>` to `Match` struct, populated only for BedAndBreakfast matches.

**Rationale**:

- B&B rule matches sale with purchase 1-30 days later
- Users need to see which purchase was matched for verification
- HMRC audits may require this traceability
- Optional field avoids clutter for SameDay/Section104 matches

**Alternatives Considered**:

| Alternative                          | Rejected Because                                          |
| ------------------------------------ | --------------------------------------------------------- |
| Always require acquisition date      | SameDay date is same as disposal; S104 has no single date |
| Separate BedAndBreakfastMatch struct | Over-engineering; enum variants complicate serialization  |

### 4. Decimal Precision in JSON Output

**Decision**: Use `rust_decimal`'s `Decimal` with custom serialization to round to 2 decimal places for JSON output.

**Rationale**:

- Current output shows floating-point artifacts: `"34.202000000000005"`
- Financial reports should show clean values: `"34.20"`
- Internal calculations continue using full precision
- Custom serializer rounds only for display

**Alternatives Considered**:

| Alternative              | Rejected Because                             |
| ------------------------ | -------------------------------------------- |
| Store pre-rounded values | Loses precision for calculations             |
| Integer pence storage    | Complicates all arithmetic                   |
| Post-process JSON        | Fragile, doesn't work with schema generators |

**Implementation Pattern**:

```rust
mod decimal_2dp {
    use rust_decimal::Decimal;
    use serde::{Deserializer, Serializer};

    pub fn serialize<S>(value: &Decimal, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&value.round_dp(2).to_string())
    }
}

#[derive(Serialize)]
pub struct Match {
    #[serde(serialize_with = "decimal_2dp::serialize")]
    pub proceeds: Decimal,
    // ...
}
```

### 5. Multi-Year Report Structure

**Decision**: `TaxReport` contains `Vec<TaxYearSummary>` where each summary has disposals and totals for that period.

**Rationale**:

- Supports single-year and multi-year views from same structure
- Single-year report is just `tax_years.len() == 1`
- Each tax year has its own totals for self-assessment
- `holdings` at top level shows end-state after all disposals

**Structure**:

```rust
pub struct TaxReport {
    pub tax_years: Vec<TaxYearSummary>,
    pub holdings: Vec<Section104Pool>,
}

pub struct TaxYearSummary {
    pub period: TaxPeriod,
    pub disposals: Vec<Disposal>,
    pub total_gain: Decimal,
    pub total_loss: Decimal,
    pub net_gain: Decimal,
}
```

## Dependencies

No new dependencies required. Using existing:

- `serde` / `serde_json` - serialization
- `chrono` - date handling
- `rust_decimal` - decimal arithmetic
- `thiserror` - error types
- `schemars` - JSON schema generation

## Risks & Mitigations

| Risk                         | Likelihood | Impact | Mitigation                                   |
| ---------------------------- | ---------- | ------ | -------------------------------------------- |
| Test migration effort        | High       | Medium | Automated script to convert JSON files       |
| Calculator refactoring scope | Medium     | Medium | Incremental changes, preserve existing logic |
| Breaking CLI output          | Low        | High   | Version flag or migration period if needed   |

## Open Questions

None - all clarifications resolved.
