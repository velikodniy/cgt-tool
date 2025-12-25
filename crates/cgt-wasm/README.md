# cgt-wasm

WebAssembly bindings for the UK Capital Gains Tax (CGT) calculator. This package enables privacy-preserving client-side tax calculations in web browsers and JavaScript/TypeScript environments.

## Features

- **Client-side computation**: All calculations run in the browser - your financial data never leaves your machine
- **JSON output**: Structured data for easy integration with web applications
- **Embedded FX rates**: All HMRC foreign exchange rates bundled (no external dependencies)
- **TypeScript support**: Auto-generated type definitions for type-safe development
- **HMRC-compliant**: Implements official UK tax rules for share matching and calculations

## Installation

### From GitHub Releases (Recommended)

Download the tarball from the latest release:

```bash
# npm
npm install https://github.com/velikodniy/cgt-tool/releases/download/v0.7.1/cgt-tool-wasm-v0.7.1.tgz

# bun
bun add https://github.com/velikodniy/cgt-tool/releases/download/v0.7.1/cgt-tool-wasm-v0.7.1.tgz

# yarn
yarn add https://github.com/velikodniy/cgt-tool/releases/download/v0.7.1/cgt-tool-wasm-v0.7.1.tgz
```

### Local Build

```bash
# Install wasm-pack
cargo install wasm-pack

# Build the package
cd crates/cgt-wasm
wasm-pack build --target web

# The output will be in crates/cgt-wasm/pkg/
```

## Usage

### Browser (ES Modules)

```html
<script type="module">
  import init, { calculate_tax, parse_transactions, validate_dsl } from './pkg/cgt_wasm.js';

  // Initialize WASM module
  await init();

  // Define transactions using the DSL
  const dsl = `
    2024-01-15 BUY AAPL 100 @ 150.00 USD FEES 10.00 USD
    2024-06-20 SELL AAPL 50 @ 180.00 USD FEES 8.00 USD
  `;

  // Calculate tax report for 2024/25 tax year
  const reportJson = calculate_tax(dsl, 2024);
  const report = JSON.parse(reportJson);

  console.log('Total gain:', report.tax_years[0].total_gain);
  console.log('Tax liability:', report.tax_years[0].tax_liability);
</script>
```

### Node.js / TypeScript

```typescript
import init, { calculate_tax, parse_transactions, validate_dsl } from 'cgt-tool-wasm';

async function main() {
  // Initialize WASM
  await init();

  const dsl = `
    2024-01-15 BUY AAPL 100 @ 150.00 USD
    2024-06-20 SELL AAPL 50 @ 180.00 USD
  `;

  // Parse transactions
  const transactionsJson = parse_transactions(dsl);
  const transactions = JSON.parse(transactionsJson);
  console.log('Parsed transactions:', transactions);

  // Validate DSL
  const validationJson = validate_dsl(dsl);
  const validation = JSON.parse(validationJson);

  if (!validation.is_valid) {
    console.error('Validation errors:', validation.errors);
    return;
  }

  // Calculate tax
  const reportJson = calculate_tax(dsl, 2024);
  const report = JSON.parse(reportJson);
  console.log('Tax report:', report);
}

main();
```

### Deno

```typescript
import init, { calculate_tax } from './pkg/cgt_wasm.js';

await init();

const dsl = "2024-01-15 BUY AAPL 10 @ 150.00 USD";
const report = JSON.parse(calculate_tax(dsl, 2024));
console.log(report);
```

## API Reference

### `calculate_tax(dsl: string, tax_year?: number | null): string`

Calculate a complete tax report from transaction DSL.

**Parameters:**

- `dsl`: Transaction DSL string (see DSL syntax below)
- `tax_year`: Optional tax year start (e.g., `2024` for 2024/25). If `null` or omitted, returns all years.

**Returns:** JSON string containing the tax report with structure:

```typescript
{
  tax_years: Array<{
    year: number,
    total_gain: string,
    total_loss: string,
    net_gain: string,
    tax_liability: string,
    // ... more fields
  }>,
  holdings: Array<{
    ticker: string,
    quantity: string,
    cost_basis: string
  }>
}
```

### `parse_transactions(dsl: string): string`

Parse transaction DSL without performing calculations.

**Parameters:**

- `dsl`: Transaction DSL string

**Returns:** JSON array of parsed transactions

### `validate_dsl(dsl: string): string`

Validate transaction DSL and return errors/warnings.

**Parameters:**

- `dsl`: Transaction DSL string

**Returns:** JSON object:

```typescript
{
  is_valid: boolean,
  errors: Array<{
    line?: number,
    date: string,
    ticker: string,
    message: string
  }>,
  warnings: Array<{
    line?: number,
    date: string,
    ticker: string,
    message: string
  }>
}
```

## DSL Syntax

The DSL (Domain-Specific Language) is a simple text format for describing stock transactions:

```
# Comments start with #

# Buy shares
YYYY-MM-DD BUY <TICKER> <QUANTITY> @ <PRICE> [CURRENCY] [FEES <FEE_AMOUNT> [CURRENCY]]

# Sell shares
YYYY-MM-DD SELL <TICKER> <QUANTITY> @ <PRICE> [CURRENCY] [FEES <FEE_AMOUNT> [CURRENCY]]

# Dividends
YYYY-MM-DD DIVIDEND <TICKER> <AMOUNT> [CURRENCY]

# Stock splits
YYYY-MM-DD SPLIT <TICKER> <NEW_SHARES> FOR <OLD_SHARES>

# Capital returns
YYYY-MM-DD CAPRETURN <TICKER> <AMOUNT_PER_SHARE> [CURRENCY]
```

**Example:**

```
2024-01-15 BUY AAPL 100 @ 150.00 USD FEES 10.00 USD
2024-03-20 BUY AAPL 50 @ 160.00 USD
2024-06-25 SELL AAPL 75 @ 180.00 USD FEES 8.00 USD
2024-09-10 DIVIDEND AAPL 50.00 USD
2024-12-01 SPLIT AAPL 4 FOR 1
```

Currency defaults to GBP if omitted. All foreign currency amounts are automatically converted to GBP using bundled HMRC exchange rates.

## FX Rates

All HMRC foreign exchange rates from January 2015 through August 2025 are embedded in the WASM binary. No external API calls or additional downloads are required.

## Bundle Size

- **Uncompressed WASM**: ~4.9 MB
- **Gzipped**: ~614 KB
- **Brotli**: ~450 KB (estimated)

The binary includes embedded FX rate data. Most web servers automatically compress WASM files when serving.

## Browser Compatibility

Requires browsers with WebAssembly support:

- Chrome 57+
- Firefox 52+
- Safari 11+
- Edge 16+

## Examples

See [examples/wasm-demo/](../../examples/wasm-demo/) for a complete browser-based calculator with UI.

## License

AGPL-3.0 - See [LICENSE](../../LICENSE) for details

## Related

- [cgt-tool](https://github.com/velikodniy/cgt-tool) - Command-line version
- [Tax Rules Documentation](../../TAX_RULES.md) - HMRC rules reference
