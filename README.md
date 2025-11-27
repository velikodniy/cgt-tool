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
cgt-cli parse transactions.txt
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
cgt-cli report transactions.txt --year 2024
```

## Input Format

One transaction per line: `YYYY-MM-DD ACTION TICKER ...`

```text
2025-04-01 BUY  AAPL 100 150.00 5.00
2025-05-01 SELL AAPL 50  160.00 5.00
2025-06-01 DIVIDEND AAPL 100 25.50
2025-07-01 CAPRETURN AAPL 100 50.00
2025-08-01 SPLIT AAPL 2.0
```

- **BUY/SELL**: `DATE ACTION TICKER AMOUNT PRICE EXPENSES`
- **DIVIDEND**: `DATE DIVIDEND TICKER AMOUNT TAX_PAID`
- **CAPRETURN**: `DATE CAPRETURN TICKER AMOUNT EXPENSES`
- **SPLIT/UNSPLIT**: `DATE SPLIT TICKER RATIO`
