# Capital Gains Tax Tool

A CLI tool to calculate UK Capital Gains Tax for share disposals using HMRC Same Day, Bed & Breakfast, and Section 104 matching rules.

The main idea is a small, line-based DSL that describes your transactions (buys, sells, dividends, splits) in a consistent format. You can write it by hand, or convert broker exports into this intermediate format and then generate CGT reports from it.

> [!WARNING]
> I am not an accountant or tax lawyer, and I do not provide consultations. Use this tool at your own risk.

> [!IMPORTANT]
> This is a 0.x project. The CLI, DSL, and outputs may change even though the calculations are carefully tested.

## Installation

### Homebrew (macOS & Linux)

```bash
brew tap velikodniy/tap
brew install cgt-tool
cgt-tool --version
```

### Pre-built binaries

Download the latest release for your platform from [GitHub Releases](https://github.com/velikodniy/cgt-tool/releases).

```bash
chmod +x cgt-tool-*
./cgt-tool-linux-x86_64 --help
```

### Build from source

Requires [Rust](https://rustup.rs/) 1.85+ (2024 edition):

```bash
cargo install --path crates/cgt-cli
```

## Quick start

Convert broker data (optional):

```bash
cgt-tool convert schwab transactions.json --output transactions.cgt
```

Or write a small `.cgt` file:

```text
2024-04-01 BUY AAPL 100 @ 150.00 USD FEES 5.00 USD
2024-06-20 SELL AAPL 50 @ 180.00 USD
```

Generate a report:

```bash
cgt-tool report transactions.cgt
```

Sample output:

```text
Tax year    Gain     Proceeds   Allowance   Taxable
2024/2025   £1,200   £7,200     £3,000      £0

SELL 50 AAPL on 20/06/2024 - GAIN £1,200
  Section 104: 50 shares @ £120.00
  Proceeds: £7,200 | Cost: £6,000 | Gain: £1,200
```

Output formats: plain text (default), PDF (`--format pdf`), JSON (`--format json`).

You can pass multiple `.cgt` files if your broker limits export ranges (e.g., Schwab exports max 4 years):

```bash
cgt-tool report part1.cgt part2.cgt --format pdf --output report.pdf
```

## What you need

- Your transactions: broker exports (preferred) or a hand-written `.cgt` file
- Enough history to cover buys before sells (especially for long-held positions)
- Fees/commissions where applicable (they affect the gain)
- For foreign assets: currency codes (otherwise values are treated as GBP)

## CLI overview

- `report`: calculate gains/losses and produce plain text, PDF, or JSON
- `convert`: convert supported broker exports into the `.cgt` DSL
- `parse`: parse/normalize `.cgt` and print JSON (debugging and scripting)
- `mcp`: run an MCP server so AI apps can call the calculator

Run `cgt-tool --help` or `cgt-tool <subcommand> --help` for all options.

## Input format (quick reference)

One transaction per line. Keywords are uppercase, placeholders are `<angle brackets>`, optional parts are `[square brackets]`.

```text
# Comment
2025-04-01 BUY AAPL 100 @ 150.00 FEES 5.00
2025-05-01 SELL AAPL 50 @ 160.00 FEES 5.00
```

<details>
<summary>Input format details</summary>

```text
BUY/SELL:    <date> BUY|SELL <ticker> <quantity> @ <price> [<currency>] [FEES <amount> [<currency>]]
DIVIDEND:    <date> DIVIDEND <ticker> <quantity> TOTAL <value> [<currency>] [TAX <amount> [<currency>]]
CAPRETURN:   <date> CAPRETURN <ticker> <quantity> TOTAL <value> [<currency>] [FEES <amount> [<currency>]]
SPLIT:       <date> SPLIT|UNSPLIT <ticker> RATIO <ratio>
```

</details>

## Currency and FX rates

- GBP is the default currency. Add ISO 4217 codes after amounts to use foreign currencies.
- The tool uses HMRC monthly average exchange rates for UK tax calculations.
- Bundled FX rates are updated regularly in releases to track the latest HMRC data.

FX rates source: [UK Trade Tariff Exchange Rates](https://www.trade-tariff.service.gov.uk/exchange_rates)

Use a custom folder with newer or missing rates:

```bash
cgt-tool report transactions.cgt --year 2024 --fx-folder ./my-rates
```

Download rates:

```bash
./scripts/download-fx-rates.sh
./scripts/download-fx-rates.sh ./my-rates
```

## Broker conversion details

### Charles Schwab

Export your data:

1. Log in to Schwab.com
2. Navigate to Accounts → History
3. Select date range and account
4. Export as JSON (transactions)
5. For RSUs: Stock Plan → Award History → Export as JSON (awards / FMV)

Convert to `.cgt`:

```bash
cgt-tool convert schwab transactions.json --output transactions.cgt
```

Supported transaction types:

- Buy/Sell transactions
- RSU vesting (Stock Plan Activity) with `--awards` Fair Market Value data
- Dividends (Cash Dividend, Qualified Dividend, Short/Long Term Cap Gain)
- Dividend tax withholding (NRA Tax Adj, NRA Withholding)
- Date formats: `MM/DD/YYYY` and `as of MM/DD/YYYY`

Unsupported/skipped:

- Wire transfers (Wire Sent/Received)
- Interest payments (Credit Interest)
- Stock splits (requires manual ratio entry)
- Other non-CGT-relevant transactions

Conversion examples:

```bash
# Basic conversion (transactions only)
cgt-tool convert schwab transactions.json

# With RSU vesting data (requires equity awards file)
cgt-tool convert schwab transactions.json --awards awards.json

# Save to file instead of stdout
cgt-tool convert schwab transactions.json --output output.cgt
```

## Troubleshooting

- If numbers look wrong, check currency: amounts without a currency code are treated as GBP.
- UK tax years run 6 April to 5 April; use `--year` if you want to focus on a specific year.
- If you are missing buys (only have recent exports), your gains may be wrong because the matching rules require earlier acquisitions.

## Advanced

### Parse subcommand (debugging / scripting)

`parse` reads `.cgt` and prints normalized JSON. It is useful for sanity-checking your input and piping into tools like `jq`.

```bash
cgt-tool parse transactions.cgt | jq
```

### WebAssembly (experimental)

For browser or Node.js usage (experimental). Download the `.tgz` from [GitHub Releases](https://github.com/velikodniy/cgt-tool/releases) and install:

```bash
npm install cgt-tool-wasm-<version>.tgz
```

See [`crates/cgt-wasm/README.md`](crates/cgt-wasm/README.md) and [`web/`](web/) for full docs and the demo.

### MCP server (AI assistant integration)

MCP (Model Context Protocol) is an open standard for connecting AI apps to external tools.

Start the server:

```bash
cgt-tool mcp
```

To connect it to clients like Claude Desktop, you typically add a small JSON config that points to the `cgt-tool` binary and passes `mcp` as an argument. See MCP’s client docs for step-by-step setup:

- [What is MCP](https://modelcontextprotocol.io/docs/getting-started/intro)
- [Connecting local servers](https://modelcontextprotocol.io/docs/develop/connect-local-servers) (includes Claude Desktop setup)

## Tax rules

For detailed CGT matching rules, see [`docs/tax-rules.md`](docs/tax-rules.md).

## Acknowledgments

Tests and ideas reused from [cgtcalc](https://github.com/mattjgalloway/cgtcalc) made this project possible.
