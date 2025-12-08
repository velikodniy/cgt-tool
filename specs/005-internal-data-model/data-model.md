# Data Model: Internal Data Model Improvements

**Feature**: 005-internal-data-model
**Date**: 2025-12-08

## Entity Overview

```text
TaxReport
├── tax_years: Vec<TaxYearSummary>
│   ├── period: TaxPeriod
│   ├── disposals: Vec<Disposal>
│   │   ├── date: NaiveDate
│   │   ├── ticker: String
│   │   ├── quantity: Decimal
│   │   ├── proceeds: Decimal
│   │   └── matches: Vec<Match>
│   │       ├── rule: MatchRule
│   │       ├── quantity: Decimal
│   │       ├── allowable_cost: Decimal
│   │       ├── gain_or_loss: Decimal
│   │       └── acquisition_date: Option<NaiveDate>
│   ├── total_gain: Decimal
│   ├── total_loss: Decimal
│   └── net_gain: Decimal
└── holdings: Vec<Section104Pool>
    ├── ticker: String
    ├── quantity: Decimal
    └── total_cost: Decimal
```

## Entities

### TaxPeriod

A validated UK tax year identifier (April 6 to April 5).

| Field      | Type  | Description                  |
| ---------- | ----- | ---------------------------- |
| (internal) | `u16` | Start year of the tax period |

**Serialization**: `"2023/24"` format (YYYY/YY)

**Validation Rules**:

- Start year must be in range 1900-2100
- Automatically derives end year as start + 1
- Rejects invalid patterns (e.g., "2023/27", "23/24", "2023-24")

**Derivation**:

- From date: If before April 6, use previous year as start
- Example: 2024-03-15 → "2023/24", 2024-04-10 → "2024/25"

### TaxYearSummary

Summary of CGT activity within a single UK tax year.

| Field      | Type            | Description                                       |
| ---------- | --------------- | ------------------------------------------------- |
| period     | `TaxPeriod`     | The tax year identifier                           |
| disposals  | `Vec<Disposal>` | All disposals in this tax year                    |
| total_gain | `Decimal`       | Sum of gains (positive gain_or_loss)              |
| total_loss | `Decimal`       | Sum of losses (negative gain_or_loss as positive) |
| net_gain   | `Decimal`       | total_gain - total_loss                           |

**Validation Rules**:

- `net_gain == total_gain - total_loss`
- All disposals must fall within the tax year date range

### Disposal

A sale event that triggers CGT calculation.

| Field    | Type         | Description                                     |
| -------- | ------------ | ----------------------------------------------- |
| date     | `NaiveDate`  | Date of the sale                                |
| ticker   | `String`     | Security identifier (ISIN or symbol)            |
| quantity | `Decimal`    | Total quantity sold                             |
| proceeds | `Decimal`    | Gross sale amount (quantity × price - expenses) |
| matches  | `Vec<Match>` | How the disposal was matched                    |

**Validation Rules**:

- Sum of match quantities must equal disposal quantity
- At least one match required

**Relationships**:

- One-to-many with `Match`
- Belongs to one `TaxYearSummary` (determined by date)

### Match

How a disposal (or portion) was matched to an acquisition.

| Field            | Type                | Description                            |
| ---------------- | ------------------- | -------------------------------------- |
| rule             | `MatchRule`         | Which matching rule applied            |
| quantity         | `Decimal`           | Quantity matched by this rule          |
| allowable_cost   | `Decimal`           | Cost basis for this match              |
| gain_or_loss     | `Decimal`           | proceeds_portion - allowable_cost      |
| acquisition_date | `Option<NaiveDate>` | Date of matched acquisition (B&B only) |

**Validation Rules**:

- If rule is `BedAndBreakfast`, acquisition_date MUST be Some
- If rule is `SameDay` or `Section104`, acquisition_date SHOULD be None
- For B&B: acquisition_date must be 1-30 days after disposal date

### MatchRule

Enumeration of HMRC share matching rules.

| Variant         | Description                                          |
| --------------- | ---------------------------------------------------- |
| SameDay         | Matched with shares bought on same day               |
| BedAndBreakfast | Matched with shares bought within 30 days after sale |
| Section104      | Matched from pooled holding at average cost          |

### Section104Pool

End-of-period state of a Section 104 pooled holding.

| Field      | Type      | Description              |
| ---------- | --------- | ------------------------ |
| ticker     | `String`  | Security identifier      |
| quantity   | `Decimal` | Shares remaining in pool |
| total_cost | `Decimal` | Total cost basis of pool |

**Derived**: `average_cost = total_cost / quantity`

### TaxReport

The complete CGT calculation output.

| Field     | Type                  | Description                            |
| --------- | --------------------- | -------------------------------------- |
| tax_years | `Vec<TaxYearSummary>` | One or more tax year summaries         |
| holdings  | `Vec<Section104Pool>` | End-state holdings after all disposals |

## State Transitions

### Disposal Matching Flow

```text
Sell Transaction
     │
     ▼
┌─────────────┐
│ Check Same  │──Yes──▶ Create Match(SameDay)
│    Day      │              │
└──────┬──────┘              │
       │ No                  │
       ▼                     │
┌─────────────┐              │
│ Check B&B   │──Yes──▶ Create Match(BedAndBreakfast)
│  (30 days)  │         + acquisition_date
└──────┬──────┘              │
       │ No                  │
       ▼                     │
┌─────────────┐              │
│  Section    │──────▶ Create Match(Section104)
│    104      │              │
└─────────────┘              │
                             ▼
                      Create Disposal
                      (with all matches)
                             │
                             ▼
                      Add to TaxYearSummary
                      (based on disposal date)
```

## JSON Schema Examples

### Current Format (Before)

```json
{
  "tax_year": 2023,
  "matches": [
    {
      "date": "2023-10-15",
      "ticker": "AAPL",
      "quantity": "100",
      "proceeds": "15234.567890123",
      "allowable_cost": "10000.00",
      "gain_or_loss": "5234.57",
      "rule": "SameDay"
    }
  ],
  "total_gain": "5234.57",
  "total_loss": "0",
  "net_gain": "5234.57",
  "holdings": []
}
```

### New Format (After)

```json
{
  "tax_years": [
    {
      "period": "2023/24",
      "disposals": [
        {
          "date": "2023-10-15",
          "ticker": "AAPL",
          "quantity": "100",
          "proceeds": "15234.57",
          "matches": [
            {
              "rule": "SameDay",
              "quantity": "100",
              "allowable_cost": "10000.00",
              "gain_or_loss": "5234.57"
            }
          ]
        }
      ],
      "total_gain": "5234.57",
      "total_loss": "0.00",
      "net_gain": "5234.57"
    }
  ],
  "holdings": []
}
```

### B&B Match Example

```json
{
  "rule": "BedAndBreakfast",
  "quantity": "50",
  "allowable_cost": "5200.00",
  "gain_or_loss": "-200.00",
  "acquisition_date": "2023-10-25"
}
```

## Migration Notes

### Breaking Changes

1. Top-level `tax_year: i32` removed → Use `tax_years[].period`
2. Top-level `matches` removed → Use `tax_years[].disposals[].matches`
3. `Match.date` and `Match.ticker` removed → Available on parent `Disposal`
4. New required fields: `Disposal.proceeds`, `Match.acquisition_date` (for B&B)

### Test File Migration

All 24 `.json` test files need updating:

1. Wrap in `tax_years` array
2. Convert `tax_year: 2023` to `period: "2023/24"`
3. Group matches into disposals
4. Move `date`/`ticker` from match to disposal
5. Add `acquisition_date` for B&B matches
6. Round decimal strings to 2dp
