# Quickstart: Multi-Currency FX Conversion

## Prerequisites

- Rust toolchain (project's stable toolchain)
- No network required at runtime; FX rates can be provided via files or use bundled rates

## Running the CLI

### Basic Usage (Bundled Rates)

```bash
# Use bundled December 2024 rates (150+ currencies)
cgt-cli report transactions.cgt --year 2024
```

### With Custom FX Rates

```bash
# Point to monthly XML files from trade-tariff.service.gov.uk
cgt-cli report transactions.cgt --year 2024 --fx-folder /path/to/xml_rates
```

If a month/currency is missing from provided files, bundled rates are used as fallback.

## Input Syntax

Add ISO 4217 currency codes after any monetary amount:

```text
# GBP (default - no code needed)
2024-01-15 BUY AAPL 100 @ 150.00 EXPENSES 5.00

# USD transaction
2024-01-15 BUY AAPL 100 @ 150.00 USD EXPENSES 10.00 USD

# Mixed currencies (price in EUR, expenses in GBP)
2024-03-01 BUY MSFT 50 @ 380.00 EUR EXPENSES 5.00

# Dividend in foreign currency
2024-06-15 DIVIDEND AAPL 100 TOTAL 250.00 USD TAX 37.50 USD
```

## Report Output

- **Plain text**: Shows GBP values with original currency in parentheses
  ```
  15/06/2024 BUY 100 AAPL @ £118.42 (150 USD) (£7.89 (10 USD) fees)
  ```
- **PDF**: Shows currency symbols for foreign amounts
- **JSON**: Full `CurrencyAmount` objects with `amount`, `currency`, and `gbp` fields

## Providing FX Rates

1. Download monthly XML files from https://www.trade-tariff.service.gov.uk/exchange_rates
2. Place them in a folder
3. Pass the folder path via `--fx-folder`

If a needed currency/month is missing from both provided files and bundled rates, the tool fails with a clear error message.

## Bundled Rates

The tool includes bundled FX rates for January 2021 through December 2025 (60 months), covering 150+ currencies per month. These are used:

- When `--fx-folder` is not specified
- As fallback when a month/currency is missing from provided files

To download updated or additional rates:

```bash
./scripts/download-fx-rates.sh              # Update bundled rates (rebuild required)
./scripts/download-fx-rates.sh ./my-rates   # Download to custom folder
```

## Testing

```bash
# Run all tests
cargo test

# Run FX-specific tests
cargo test -p cgt-fx
```
