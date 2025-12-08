# Quickstart: Internal Data Model Improvements

**Feature**: 005-internal-data-model
**Date**: 2025-12-08

## Overview

This feature refactors the internal data model to better represent UK CGT domain concepts. The changes are primarily in `crates/cgt-core/src/models.rs` with cascading updates to the calculator and test files.

## Prerequisites

- Rust toolchain (2024 edition)
- Familiarity with existing `Match` and `TaxReport` structures
- Understanding of UK CGT matching rules (Same Day, B&B, Section 104)

## Key Files to Modify

| File                                | Changes                                                     |
| ----------------------------------- | ----------------------------------------------------------- |
| `crates/cgt-core/src/models.rs`     | Add TaxPeriod, Disposal, TaxYearSummary; refactor TaxReport |
| `crates/cgt-core/src/calculator.rs` | Update to produce new model structure                       |
| `crates/cgt-core/src/error.rs`      | Add InvalidTaxYear error variant                            |
| `tests/data/*.json`                 | Migrate all 24 files to new format                          |

## Implementation Order

### Step 1: Add TaxPeriod Type

```rust
// models.rs
use thiserror::Error;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
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
        let day = date.day();
        // UK tax year starts April 6
        if month < 4 || (month == 4 && day < 6) {
            Self(year - 1)
        } else {
            Self(year)
        }
    }

    pub fn start_year(&self) -> u16 {
        self.0
    }

    pub fn end_year(&self) -> u16 {
        self.0 + 1
    }
}
```

### Step 2: Custom Serialization for TaxPeriod

```rust
impl Serialize for TaxPeriod {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let end_short = (self.0 + 1) % 100;
        serializer.serialize_str(&format!("{}/{:02}", self.0, end_short))
    }
}

impl<'de> Deserialize<'de> for TaxPeriod {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        // Parse "2023/24" format
        let parts: Vec<&str> = s.split('/').collect();
        if parts.len() != 2 {
            return Err(serde::de::Error::custom("invalid tax period format"));
        }
        let start: u16 = parts[0].parse().map_err(serde::de::Error::custom)?;
        let end_short: u16 = parts[1].parse().map_err(serde::de::Error::custom)?;

        // Validate consecutive years
        let expected_end = (start + 1) % 100;
        if end_short != expected_end {
            return Err(serde::de::Error::custom("tax years must be consecutive"));
        }

        TaxPeriod::new(start).map_err(serde::de::Error::custom)
    }
}
```

### Step 3: Add Disposal and Update Match

```rust
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, JsonSchema)]
pub struct Disposal {
    pub date: NaiveDate,
    pub ticker: String,
    pub quantity: Decimal,
    pub proceeds: Decimal,
    pub matches: Vec<Match>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, JsonSchema)]
pub struct Match {
    pub rule: MatchRule,
    pub quantity: Decimal,
    pub allowable_cost: Decimal,
    pub gain_or_loss: Decimal,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub acquisition_date: Option<NaiveDate>,
}
```

### Step 4: Add TaxYearSummary and Update TaxReport

```rust
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, JsonSchema)]
pub struct TaxYearSummary {
    pub period: TaxPeriod,
    pub disposals: Vec<Disposal>,
    pub total_gain: Decimal,
    pub total_loss: Decimal,
    pub net_gain: Decimal,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, JsonSchema)]
pub struct TaxReport {
    pub tax_years: Vec<TaxYearSummary>,
    pub holdings: Vec<Section104Pool>,
}
```

### Step 5: Update Calculator

The calculator needs to:

1. Group matches by disposal date+ticker to create `Disposal` objects
2. Group disposals by tax year to create `TaxYearSummary` objects
3. Track acquisition dates for B&B matches

### Step 6: Migrate Test Files

Each `.json` file needs conversion. Example transformation:

**Before** (`Simple.json`):

```json
{
  "tax_year": 2018,
  "matches": [
    {
      "date": "2018-08-28",
      "ticker": "GB00B41YBW71",
      "quantity": "10",
      "proceeds": "34.202000000000005",
      "allowable_cost": "54.065000000000005",
      "gain_or_loss": "-20",
      "rule": "SameDay"
    }
  ],
  "total_gain": "0",
  "total_loss": "20",
  "net_gain": "-20",
  "holdings": []
}
```

**After** (`Simple.json`):

```json
{
  "tax_years": [
    {
      "period": "2018/19",
      "disposals": [
        {
          "date": "2018-08-28",
          "ticker": "GB00B41YBW71",
          "quantity": "10",
          "proceeds": "34.20",
          "matches": [
            {
              "rule": "SameDay",
              "quantity": "10",
              "allowable_cost": "54.07",
              "gain_or_loss": "-19.87"
            }
          ]
        }
      ],
      "total_gain": "0.00",
      "total_loss": "19.87",
      "net_gain": "-19.87"
    }
  ],
  "holdings": []
}
```

## Testing

```bash
# Run all tests (will fail until migration complete)
cargo test

# Run specific model tests
cargo test --package cgt-core models

# Check serialization
cargo test --package cgt-core tax_period_serialization
```

## Validation Checklist

- [ ] TaxPeriod serializes to "2023/24" format
- [ ] TaxPeriod rejects "2023/27" during deserialization
- [ ] B&B matches include acquisition_date
- [ ] SameDay/Section104 matches have no acquisition_date
- [ ] Decimal values serialize to 2 decimal places
- [ ] All 24 test files migrated
- [ ] All existing tests pass
