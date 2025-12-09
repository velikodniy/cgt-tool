# Data Model: Plain Text Report Formatter

**Feature**: 007-plain-formatter
**Date**: 2025-12-08

## Existing Entities (from cgt-core)

The formatter consumes existing data models from `cgt-core`. No modifications needed.

### TaxReport

The primary input to the formatter.

| Field     | Type                    | Description                |
| --------- | ----------------------- | -------------------------- |
| tax_years | Vec\<TaxYearSummary>    | List of tax year summaries |
| holdings  | Vec\<Section104Holding> | End-state holdings         |

### TaxYearSummary

| Field      | Type           | Description                           |
| ---------- | -------------- | ------------------------------------- |
| period     | TaxPeriod      | Tax year identifier (e.g., "2023/24") |
| disposals  | Vec\<Disposal> | List of sales in this year            |
| total_gain | Decimal        | Sum of positive gains                 |
| total_loss | Decimal        | Sum of losses (absolute value)        |
| net_gain   | Decimal        | total_gain - total_loss               |

### Disposal

| Field    | Type        | Description                        |
| -------- | ----------- | ---------------------------------- |
| date     | NaiveDate   | Date of sale                       |
| ticker   | String      | Asset identifier                   |
| quantity | Decimal     | Number of units sold               |
| proceeds | Decimal     | Sale proceeds after expenses       |
| matches  | Vec\<Match> | How acquisition costs were matched |

### Match

| Field            | Type               | Description                             |
| ---------------- | ------------------ | --------------------------------------- |
| rule             | MatchRule          | SameDay, BedAndBreakfast, or Section104 |
| quantity         | Decimal            | Units matched                           |
| allowable_cost   | Decimal            | Cost basis for matched units            |
| gain_or_loss     | Decimal            | Proceeds portion - allowable_cost       |
| acquisition_date | Option\<NaiveDate> | For B&B matches only                    |

### Section104Holding

| Field      | Type    | Description      |
| ---------- | ------- | ---------------- |
| ticker     | String  | Asset identifier |
| quantity   | Decimal | Units held       |
| total_cost | Decimal | Total cost basis |

### Transaction (for TRANSACTIONS section)

| Field     | Type      | Description                               |
| --------- | --------- | ----------------------------------------- |
| date      | NaiveDate | Transaction date                          |
| ticker    | String    | Asset identifier                          |
| operation | Operation | Buy/Sell/Dividend/CapReturn/Split/Unsplit |

## New Entities (cgt-formatter-plain)

### OutputFormat (in cgt-cli)

Enum for CLI format selection.

```rust
#[derive(ValueEnum, Clone, Default)]
pub enum OutputFormat {
    #[default]
    Plain,
    Json,
}
```

### TaxExemption (in cgt-formatter-plain)

Lookup for annual CGT exemption values.

```rust
pub fn get_exemption(tax_year_start: u16) -> Decimal
```

| Tax Year Start | Exemption |
| -------------- | --------- |
| 2018           | £11,700   |
| 2019           | £12,000   |
| 2020           | £12,300   |
| 2021           | £12,300   |
| 2022           | £12,300   |
| 2023           | £6,000    |
| 2024           | £3,000    |

## Data Flow

```text
.cgt file
    ↓ parse_file()
Vec<Transaction>
    ↓ calculate()
TaxReport
    ↓ format() [with Vec<Transaction>]
String (plain text)
```

## Format Sections Mapping

| Section                | Data Source                                       |
| ---------------------- | ------------------------------------------------- |
| SUMMARY                | TaxYearSummary + TaxExemption lookup              |
| TAX YEAR DETAILS       | Disposal + Match                                  |
| TAX RETURN INFORMATION | TaxYearSummary (aggregated)                       |
| HOLDINGS               | Section104Holding                                 |
| TRANSACTIONS           | Transaction (original input)                      |
| ASSET EVENTS           | Transaction (Dividend, CapReturn, Split, Unsplit) |
