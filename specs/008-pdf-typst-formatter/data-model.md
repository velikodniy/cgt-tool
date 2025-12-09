# Data Model: PDF Typst Formatter

## Existing Entities (from cgt-core)

The PDF formatter reuses existing data structures from `cgt-core`. No new persistent entities are introduced.

### TaxReport

The primary input to the formatter. Contains:

- `tax_years: Vec<TaxYearSummary>` - Summary per tax year
- `holdings: Vec<Section104Holding>` - Remaining positions

### TaxYearSummary

- `period: TaxPeriod` - Tax year (e.g., 2023/2024)
- `total_gain: Decimal` - Sum of gains
- `total_loss: Decimal` - Sum of losses
- `net_gain: Decimal` - Net result
- `disposals: Vec<Disposal>` - Individual sales

### Disposal

- `date: NaiveDate` - Sale date
- `ticker: String` - Asset identifier
- `quantity: Decimal` - Shares sold
- `proceeds: Decimal` - Sale proceeds
- `matches: Vec<Match>` - How shares were matched

### Match

- `rule: MatchRule` - Same Day, B&B, or Section 104
- `quantity: Decimal` - Shares matched
- `allowable_cost: Decimal` - Cost basis
- `gain_or_loss: Decimal` - Result
- `acquisition_date: Option<NaiveDate>` - For B&B matches

### Transaction

Input transactions passed to formatter for display:

- `date: NaiveDate`
- `ticker: String`
- `operation: Operation` - Buy, Sell, Dividend, etc.

## New Types (in cgt-formatter-pdf)

### PdfOptions

Configuration for PDF generation:

```rust
pub struct PdfOptions {
    /// Paper size (default: A4)
    pub paper: PaperSize,
    /// Page margins in cm (default: 2.0)
    pub margin_cm: f32,
}

pub enum PaperSize {
    A4,
    Letter,
}

impl Default for PdfOptions {
    fn default() -> Self {
        Self {
            paper: PaperSize::A4,
            margin_cm: 2.0,
        }
    }
}
```

## Data Flow

```text
┌─────────────┐    ┌──────────────────┐    ┌─────────────┐
│  TaxReport  │───▶│ cgt-formatter-pdf│───▶│  Vec<u8>    │
│ Transaction │    │                  │    │  (PDF bytes)│
└─────────────┘    └──────────────────┘    └─────────────┘
                           │
                           ▼
                   ┌──────────────────┐
                   │  Typst Template  │
                   │  (embedded)      │
                   └──────────────────┘
```

## PDF Document Structure

The generated PDF follows this logical structure:

```text
┌────────────────────────────────────────┐
│ HEADER                                 │
│ Capital Gains Tax Report               │
│ Tax Year: 2023/2024                    │
│ Generated: 09/12/2024                  │
├────────────────────────────────────────┤
│ SUMMARY TABLE                          │
│ ┌──────────┬───────────┬─────────────┐ │
│ │ Metric   │ Value     │ Notes       │ │
│ ├──────────┼───────────┼─────────────┤ │
│ │ Gain     │ £1,234    │             │ │
│ │ Proceeds │ £10,000   │             │ │
│ │ Exemption│ £6,000    │             │ │
│ │ Taxable  │ £0        │             │ │
│ └──────────┴───────────┴─────────────┘ │
├────────────────────────────────────────┤
│ DISPOSAL DETAILS                       │
│ 1) SELL 100 AAPL on 01/05/2023         │
│    Same Day: 100 shares                │
│    Proceeds: £8,000                    │
│    Cost: £6,766                        │
│    Gain: £1,234                        │
├────────────────────────────────────────┤
│ HOLDINGS                               │
│ MSFT: 50 units @ £150.00 avg cost     │
├────────────────────────────────────────┤
│ TRANSACTIONS                           │
│ ┌──────────┬─────┬───────┬───────────┐ │
│ │ Date     │ Type│ Ticker│ Amount    │ │
│ ├──────────┼─────┼───────┼───────────┤ │
│ │ 01/01/23 │ BUY │ AAPL  │ 100@£67.66│ │
│ │ 01/05/23 │ SELL│ AAPL  │ 100@£80.00│ │
│ └──────────┴─────┴───────┴───────────┘ │
└────────────────────────────────────────┘
```

## Validation Rules

- All currency values must be formatted with £ symbol
- Dates must use UK format DD/MM/YYYY
- Negative values (losses) displayed with minus sign
- Tables must fit within A4 margins
- Long ticker symbols should not break layout
