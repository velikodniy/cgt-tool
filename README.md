# Capital Gains Tax (CGT) CLI Tool

A CLI tool to calculate Capital Gains Tax for UK assets using the "Same Day", "Bed & Breakfast", and "Section 104" matching rules.

## Installation

```bash
cargo install --path crates/cgt-cli
```

## Usage

### Parse Transactions

Verify your input file is parsed correctly:

```bash
cgt-cli parse transactions.cgt
```

Output (JSON):

```json
[
  {
    "date": "2025-04-01",
    "ticker": "AAPL",
    "action": "BUY",
    "amount": "100",
    "price": "150.00",
    "expenses": "5.00"
  }
]
```

### Generate Tax Report

Calculate gains and losses for a specific tax year:

```bash
cgt-cli report transactions.cgt --year 2024
```

Output formats: `--format plain` (default), `--format json`, or `--format pdf`

### Generate PDF Report

Generate a professional PDF document for tax reporting:

```bash
cgt-cli report transactions.cgt --year 2024 --format pdf
```

By default, the PDF is saved to `transactions.pdf`. Use `--output` for a custom path:

```bash
cgt-cli report transactions.cgt --year 2024 --format pdf --output report.pdf
```

Example plain text output:

```text
# SUMMARY

Tax year    Gain   Proceeds   Exemption   Taxable gain
==========================================================
2024/2025   £500   £8000      £3000       £0

# TAX YEAR DETAILS

## 2024/2025

1) SELL 50 AAPL on 01/05/2025 - GAIN £500
   Section 104: 50 shares @ £150
   Proceeds: 50 × £160 = £8000
   Cost: £7500
   Result: £500

# HOLDINGS

AAPL: 100 units at £152.5 avg cost

# TRANSACTIONS

01/04/2025 BUY 100 AAPL @ £150 (£5 fees)
01/04/2025 BUY 50 AAPL @ £155 (£0 fees)
01/05/2025 SELL 50 AAPL @ £160 (£5 fees)
```

## Input Format

One transaction per line. Format: `YYYY-MM-DD ACTION TICKER AMOUNT @ PRICE [EXPENSES EXPENSE_AMOUNT]`

```text
# This is a comment and will be ignored
2025-04-01 BUY AAPL 100 @ 150.00 EXPENSES 5.00
2025-04-01 BUY AAPL 50 @ 155.00 # Expenses are optional
2025-05-01 SELL AAPL 50 @ 160.00 EXPENSES 5.00
```

- **BUY/SELL**: `YYYY-MM-DD ACTION TICKER AMOUNT @ PRICE [EXPENSES EXPENSE_AMOUNT]`
- **DIVIDEND**: `YYYY-MM-DD DIVIDEND TICKER AMOUNT TAX TAX_AMOUNT`
- **CAPRETURN**: `YYYY-MM-DD CAPRETURN TICKER AMOUNT EXPENSES EXPENSE_AMOUNT`
- **SPLIT/UNSPLIT**: `YYYY-MM-DD SPLIT FOO RATIO RATIO_VALUE`

## Tax Rules Documentation

For detailed information about UK CGT share matching rules, see [TAX_RULES.md](./TAX_RULES.md).

## Acknowledgments

The test suite for this project was developed using test cases from [cgtcalc](https://github.com/mattjgalloway/cgtcalc) by Matt Galloway. We are grateful for this reference implementation which helped validate our CGT calculations against UK tax rules.

**Test Data Attribution:**

- Source: https://github.com/mattjgalloway/cgtcalc
- Commit: 896d91486805e27fcea0e851ee01868b86e161f5
- Date: 2025-11-21

The test cases have been adapted to our DSL format while preserving the calculation scenarios and expected results
