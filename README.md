# Capital Gains Tax (CGT) CLI Tool

A CLI tool to calculate Capital Gains Tax for UK assets using the "Same Day", "Bed & Breakfast", and "Section 104" matching rules.

## Installation

### Pre-built Binaries (Recommended)

Download the latest release for your platform from [GitHub Releases](https://github.com/vadim-projects/cgt-tool/releases):

| Platform                   | Download                      |
| -------------------------- | ----------------------------- |
| Linux (x86_64)             | `cgt-tool-linux-x86_64`       |
| Linux (ARM64/Raspberry Pi) | `cgt-tool-linux-aarch64`      |
| macOS (Intel)              | `cgt-tool-macos-x86_64`       |
| macOS (Apple Silicon)      | `cgt-tool-macos-aarch64`      |
| Windows (x86_64)           | `cgt-tool-windows-x86_64.exe` |

After downloading, make the binary executable (Linux/macOS):

```bash
chmod +x cgt-tool-*
./cgt-tool-linux-x86_64 --help
```

Optionally, move it to a directory in your PATH:

```bash
sudo mv cgt-tool-linux-x86_64 /usr/local/bin/cgt-tool
```

### Build from Source

Requires [Rust](https://rustup.rs/) 1.85+ (2024 edition):

```bash
cargo install --path crates/cgt-cli
```

Or build manually:

```bash
cargo build --release -p cgt-cli
./target/release/cgt-tool --help
```

## Usage

### Convert Broker Exports

Convert broker export files to CGT DSL format:

#### Charles Schwab

```bash
# Basic conversion (transactions only)
cgt-tool convert schwab transactions.csv

# With RSU vesting data (requires equity awards file)
cgt-tool convert schwab transactions.csv --awards awards.json

# Save to file instead of stdout
cgt-tool convert schwab transactions.csv --output output.cgt
```

**Supported Schwab transaction types:**

- Buy/Sell transactions
- RSU vesting (Stock Plan Activity) - requires `--awards` file with Fair Market Value data
- Dividends (Cash Dividend, Qualified Dividend, Short/Long Term Cap Gain)
- Dividend tax withholding (NRA Tax Adj, NRA Withholding)
- Date formats: `MM/DD/YYYY` and `as of MM/DD/YYYY`

**Unsupported/skipped transaction types:**

- Wire transfers (Wire Sent/Received)
- Interest payments (Credit Interest)
- Stock splits (requires manual ratio entry)
- Other non-CGT-relevant transactions

**Getting Schwab export files:**

1. Log in to Schwab.com
2. Navigate to Accounts → History
3. Select date range and account
4. Export as CSV (for transactions)
5. For RSUs: Navigate to Stock Plan → Award History → Export as JSON

**Example output:**

```text
# Converted from Charles Schwab export
# Source files: transactions.csv
# Converted: 2025-12-13T19:12:32.513258+00:00

2023-04-25 BUY GOOG 10 @ 125.50 USD EXPENSES 4.95 USD
2023-05-10 SELL GOOG 5 @ 130.00 USD EXPENSES 2.50 USD
```

### Parse Transactions

Verify your input file is parsed correctly:

```bash
cgt-tool parse transactions.cgt
```

Output (JSON):

```json
[
  {
    "date": "2025-04-01",
    "ticker": "AAPL",
    "action": "BUY",
    "amount": "100",
    "price": {
      "amount": "150.00",
      "currency": "GBP",
      "gbp": "150.00"
    },
    "expenses": {
      "amount": "5.00",
      "currency": "GBP",
      "gbp": "5.00"
    }
  }
]
```

### Generate Tax Report

Calculate gains and losses for a specific tax year:

```bash
cgt-tool report transactions.cgt --year 2024
```

Output formats: `--format plain` (default), `--format json`, or `--format pdf`

### Generate PDF Report

Generate a professional PDF document for tax reporting:

```bash
cgt-tool report transactions.cgt --year 2024 --format pdf
```

By default, the PDF is saved to `transactions.pdf`. Use `--output` for a custom path:

```bash
cgt-tool report transactions.cgt --year 2024 --format pdf --output report.pdf
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

One transaction per line. Format: `YYYY-MM-DD ACTION TICKER AMOUNT @ PRICE [CURRENCY] [EXPENSES EXPENSE_AMOUNT [CURRENCY]]`

```text
# This is a comment and will be ignored
2025-04-01 BUY AAPL 100 @ 150.00 EXPENSES 5.00
2025-04-01 BUY AAPL 50 @ 155.00 # Expenses are optional
2025-05-01 SELL AAPL 50 @ 160.00 EXPENSES 5.00
```

- **BUY/SELL**: `YYYY-MM-DD ACTION TICKER AMOUNT @ PRICE [CURRENCY] [EXPENSES EXPENSE_AMOUNT [CURRENCY]]`
- **DIVIDEND**: `YYYY-MM-DD DIVIDEND TICKER AMOUNT TOTAL VALUE [CURRENCY] TAX TAX_AMOUNT [CURRENCY]`
- **CAPRETURN**: `YYYY-MM-DD CAPRETURN TICKER AMOUNT TOTAL VALUE [CURRENCY] EXPENSES EXPENSE_AMOUNT [CURRENCY]`
- **SPLIT/UNSPLIT**: `YYYY-MM-DD SPLIT TICKER RATIO RATIO_VALUE`

## Multi-Currency Support

The tool supports transactions in foreign currencies. Amounts are automatically converted to GBP using HMRC exchange rates for UK tax calculations.

### Syntax

Add a 3-letter ISO 4217 currency code after any monetary amount:

```text
# Buy US shares in USD
2024-06-15 BUY AAPL 100 @ 150.00 USD EXPENSES 10.00 USD

# Receive dividend in EUR
2024-09-01 DIVIDEND MSFT 50 TOTAL 125.00 EUR TAX 18.75 EUR

# Mix currencies (price in USD, expenses in GBP)
2024-10-01 BUY TSLA 10 @ 250.00 USD EXPENSES 5.00
```

If no currency code is specified, GBP is assumed.

### Exchange Rates

The tool uses HMRC monthly average exchange rates for currency conversion.

**Bundled rates**: The tool includes rates for 150+ currencies covering January 2015 through August 2025 (latest published HMRC monthly XMLs at build time). These are embedded at compile time and require no additional setup.

**Custom rates**: To use additional or updated rates, provide a folder containing XML files:

```bash
cgt-tool report transactions.cgt --year 2024 --fx-folder ./my-rates
```

**Downloading rates**: Use the included script to download missing or updated rates:

```bash
# Download missing rates to the bundled folder (requires rebuild)
./scripts/download-fx-rates.sh

# Download to a custom folder (no rebuild needed)
./scripts/download-fx-rates.sh ./my-rates
```

**Rate source**: Monthly XML files are available from the UK Government. FX parsing is IO-free and WASM-friendly; `cgt-tool` handles reading XML files and passes their contents into the parser:

- Current rates (2021+): https://www.trade-tariff.service.gov.uk/exchange_rates
- API: `https://www.trade-tariff.service.gov.uk/api/v2/exchange_rates/files/monthly_xml_YYYY-MM.xml`
- Historical rates (pre-2021): https://webarchive.nationalarchives.gov.uk/ukgwa/20231016190054/https://www.gov.uk/government/collections/exchange-rates-for-customs-and-vat

Files should be named `YYYY-MM.xml` (e.g., `2024-12.xml`); a `monthly_xml_YYYY-MM.xml` prefix also works. The CLI reads the folder, passes XML strings into the FX parser, and enforces that the embedded `<Period>` matches the file's year/month.

**Note**: The bundled rates cover January 2015 through August 2025. For transactions before 2015, you'll need to manually download historical rates from the National Archives (requires browser access) and place them in a custom `--fx-folder`.

### Report Output

- **Plain text**: Shows GBP values with original currency in parentheses when applicable
- **PDF**: Shows currency symbols (e.g., $, €) for foreign amounts
- **JSON**: Includes full `CurrencyAmount` objects with `amount`, `currency`, and `gbp` fields

Example plain text output with foreign currency:

```text
# TRANSACTIONS

15/06/2024 BUY 100 AAPL @ £118.42 (150 USD) (£7.89 (10 USD) fees)
```

## Tax Rules Documentation

For detailed information about UK CGT share matching rules, see [TAX_RULES.md](./TAX_RULES.md).

## Acknowledgments

The test suite for this project was developed using test cases from [cgtcalc](https://github.com/mattjgalloway/cgtcalc) by Matt Galloway. We are grateful for this reference implementation which helped validate our CGT calculations against UK tax rules.

**Test Data Attribution:**

- Source: https://github.com/mattjgalloway/cgtcalc
- Commit: 896d91486805e27fcea0e851ee01868b86e161f5
- Date: 2025-11-21

The test cases have been adapted to our DSL format while preserving the calculation scenarios and expected results
