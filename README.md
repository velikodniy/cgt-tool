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
