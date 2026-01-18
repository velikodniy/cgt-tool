# Capital Gains Tax (CGT) Tool

A CLI tool and WebAssembly library to calculate Capital Gains Tax for UK assets using the "Same Day", "Bed & Breakfast", and "Section 104" matching rules.

## Features

- **CLI**: Command-line interface for calculating tax reports (plain text, JSON, or PDF)
- **WebAssembly**: Run calculations in your browser with complete privacy (no server-side processing)
- **HMRC-compliant**: Implements official UK tax matching rules
- **Multi-currency**: Automatic FX conversion using bundled HMRC rates (2015-2025)

## Installation

### Homebrew (macOS & Linux)

```bash
brew tap velikodniy/tap
brew install cgt-tool
cgt-tool --version
```

### Pre-built Binaries (Recommended)

Download the latest release for your platform from [GitHub Releases](https://github.com/velikodniy/cgt-tool/releases):

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

### WebAssembly (Browser/Node.js)

For privacy-preserving client-side calculations in web browsers:

```bash
# Install from GitHub Releases
npm install https://github.com/velikodniy/cgt-tool/releases/download/v0.8.0/cgt-tool-wasm-v0.8.0.tgz
```

**Browser usage:**

```javascript
import init, { calculate_tax } from 'cgt-tool-wasm';

await init();

const dsl = `
  2024-01-15 BUY AAPL 100 @ 150.00 USD
  2024-06-20 SELL AAPL 50 @ 180.00 USD
`;

const report = JSON.parse(calculate_tax(dsl, 2024));
console.log('Total gain:', report.tax_years[0].total_gain);
```

See [crates/cgt-wasm/README.md](crates/cgt-wasm/README.md) for complete WebAssembly documentation and [web/](web/) for a live browser demo.

## CLI Usage

### Convert Broker Exports

Convert broker export files to CGT DSL format:

#### Charles Schwab

```bash
# Basic conversion (transactions only)
cgt-tool convert schwab transactions.json

# With RSU vesting data (requires equity awards file)
cgt-tool convert schwab transactions.json --awards awards.json

# Save to file instead of stdout
cgt-tool convert schwab transactions.json --output output.cgt
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
4. Export as JSON (for transactions)
5. For RSUs: Navigate to Stock Plan → Award History → Export as JSON

**Example output:**

```text
# Converted from Charles Schwab export
# Source files: transactions.json
# Converted: 2025-12-13T19:12:32.513258+00:00

2023-04-25 BUY GOOG 10 @ 125.50 USD FEES 4.95 USD
2023-05-10 SELL GOOG 5 @ 130.00 USD FEES 2.50 USD
```

### Parse Transactions

Verify your input file is parsed correctly:

```bash
cgt-tool parse transactions.cgt
```

You can also parse multiple files at once:

```bash
cgt-tool parse file1.cgt file2.cgt file3.cgt
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
    "fees": "5.00"
  }
]
```

For foreign currency transactions:

```json
[
  {
    "date": "2025-04-01",
    "ticker": "AAPL",
    "action": "BUY",
    "amount": "100",
    "price": { "amount": "150.00", "currency": "USD" },
    "fees": { "amount": "5.00", "currency": "USD" }
  }
]
```

### Generate Tax Report

Calculate gains and losses for a specific tax year:

```bash
cgt-tool report transactions.cgt --year 2024
```

Or generate a report for **all tax years** with disposals:

```bash
cgt-tool report transactions.cgt
```

When `--year` is omitted, the report includes all tax years that contain disposals, sorted chronologically. This is useful for reviewing multi-year transaction history or comparing with other CGT tools.

You can combine multiple input files (e.g., separate files per broker or year):

```bash
cgt-tool report broker1.cgt broker2.cgt --year 2024
```

Output formats: `--format plain` (default), `--format json`, or `--format pdf`

### Generate PDF Report

Generate a professional PDF document for tax reporting:

```bash
cgt-tool report transactions.cgt --year 2024 --format pdf
```

By default, the PDF is saved to `<input>.pdf` (or `report.pdf` when using multiple input files). Use `--output` for a custom path:

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

One transaction per line. Keywords are shown in uppercase, placeholders in `<angle brackets>`, optional parts in `[square brackets]`.

```text
# This is a comment and will be ignored
2025-04-01 BUY AAPL 100 @ 150.00 FEES 5.00
2025-04-01 BUY AAPL 50 @ 155.00 # Fees are optional
2025-05-01 SELL AAPL 50 @ 160.00 FEES 5.00
```

- **BUY/SELL**: `<date> BUY|SELL <ticker> <quantity> @ <price> [<currency>] [FEES <amount> [<currency>]]`
- **DIVIDEND**: `<date> DIVIDEND <ticker> <quantity> TOTAL <value> [<currency>] [TAX <amount> [<currency>]]`
- **CAPRETURN**: `<date> CAPRETURN <ticker> <quantity> TOTAL <value> [<currency>] [FEES <amount> [<currency>]]`
- **SPLIT/UNSPLIT**: `<date> SPLIT|UNSPLIT <ticker> RATIO <ratio>`

## Multi-Currency Support

The tool supports transactions in foreign currencies. Amounts are automatically converted to GBP using HMRC exchange rates for UK tax calculations.

### Syntax

Add a 3-letter ISO 4217 currency code after any monetary amount:

```text
# Buy US shares in USD
2024-06-15 BUY AAPL 100 @ 150.00 USD FEES 10.00 USD

# Receive dividend in EUR (TAX clause is optional, defaults to 0)
2024-09-01 DIVIDEND MSFT 50 TOTAL 125.00 EUR TAX 18.75 EUR

# Mix currencies (price in USD, expenses in GBP)
2024-10-01 BUY TSLA 10 @ 250.00 USD FEES 5.00
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
- **JSON**: Includes `CurrencyAmount` values - plain strings for GBP or objects with `amount` and `currency` for foreign currencies

Example plain text output with foreign currency:

```text
# TRANSACTIONS

15/06/2024 BUY 100 AAPL @ £118.42 (150 USD) (£7.89 (10 USD) fees)
```

## Tax Rules Documentation

For detailed information about UK CGT share matching rules, see [docs/tax-rules.md](docs/tax-rules.md).

## MCP Server (AI Assistant Integration)

The tool includes an MCP (Model Context Protocol) server that enables AI assistants like Claude to perform CGT calculations directly.

### Starting the Server

```bash
cgt-tool mcp
```

The server communicates over stdio using the MCP protocol.

### Available Tools

| Tool                 | Description                                                   |
| -------------------- | ------------------------------------------------------------- |
| `parse_transactions` | Parse CGT DSL or JSON transactions and return normalized JSON |
| `calculate_report`   | Generate a CGT report for a specific UK tax year              |
| `explain_matching`   | Explain how a disposal was matched using HMRC rules           |
| `get_fx_rate`        | Get HMRC exchange rate for a currency and month               |
| `convert_to_dsl`     | Convert JSON transactions to DSL format for CLI use           |

### Available Resources

| Resource   | URI                     | Description                             |
| ---------- | ----------------------- | --------------------------------------- |
| Tax Rules  | `cgt://docs/tax-rules`  | HMRC share matching rules documentation |
| DSL Syntax | `cgt://docs/dsl-syntax` | CGT DSL transaction format reference    |

### Claude Desktop Configuration

Add the following to your Claude Desktop configuration file:

**macOS**: `~/Library/Application Support/Claude/claude_desktop_config.json`
**Windows**: `%APPDATA%\Claude\claude_desktop_config.json`

```json
{
  "mcpServers": {
    "cgt-tool": {
      "command": "/path/to/cgt-tool",
      "args": ["mcp"]
    }
  }
}
```

Replace `/path/to/cgt-tool` with the actual path to the `cgt-tool` binary.

### Adding Resources in Claude Desktop

**Note**: MCP resources must be manually added in Claude Desktop to be available. To add resources:

1. Open Claude Desktop settings
2. Navigate to the MCP server configuration
3. Add the resource URIs you want to use:
   - `cgt://docs/tax-rules` - HMRC share matching rules
   - `cgt://docs/dsl-syntax` - DSL format reference

Without adding resources manually, Claude will only have access to the tools (which include schema documentation in their descriptions).

### Transaction JSON Format

When using the MCP tools, transactions can be provided as JSON:

```json
[
  {
    "date": "2024-01-15",
    "ticker": "AAPL",
    "action": "BUY",
    "amount": "100",
    "price": {"amount": "185.50", "currency": "USD"},
    "fees": {"amount": "10", "currency": "USD"}
  },
  {
    "date": "2024-06-20",
    "ticker": "AAPL",
    "action": "SELL",
    "amount": "50",
    "price": {"amount": "200", "currency": "USD"}
  }
]
```

**Important**: For US stocks, always specify `"currency": "USD"`. Without currency, amounts are treated as GBP.

### Example Prompts

Once configured, you can ask Claude questions like:

- "Parse this CGT file and show me the transactions"
- "Calculate my capital gains for tax year 2024/25"
- "Explain how this disposal was matched under HMRC rules"
- "What's the USD to GBP exchange rate for January 2024?"
- "Convert these JSON transactions to DSL format"

## Acknowledgments

The test suite for this project was developed using test cases from [cgtcalc](https://github.com/mattjgalloway/cgtcalc) by Matt Galloway. We are grateful for this reference implementation which helped validate our CGT calculations against UK tax rules.

**Test Data Attribution:**

- Source: https://github.com/mattjgalloway/cgtcalc
- Commit: 896d91486805e27fcea0e851ee01868b86e161f5
- Date: 2025-11-21

The test cases have been adapted to our DSL format while preserving the calculation scenarios and expected results
