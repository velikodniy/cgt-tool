# CGT Calculator - WASM Demo

Web interface for the UK Capital Gains Tax calculator, compiled to WebAssembly.

## File Structure

```
web/
├── index.html      # HTML structure
├── styles.css      # Styling
├── app.js          # Application logic
└── pkg/            # WASM compiled files (symlink to crates/cgt-wasm/pkg)
    ├── cgt_wasm.js
    └── cgt_wasm_bg.wasm
```

## Development

### Build WASM Module

From the root of the repository:

```bash
cd crates/cgt-wasm
wasm-pack build --target web
```

### Run Locally

Since this uses ES6 modules, you need a local server:

```bash
# Using Python 3
python3 -m http.server 8000

# Using Node.js
npx serve
```

Then open http://localhost:8000 in your browser.

## Browser Requirements

- ES6 module support
- WebAssembly support
- BigInt support

## Report data

Generate Report uses the WASM `calculate_display` function.
Rust prepares amounts, unit costs, relevant sections, and notes before the browser arranges the HTML.
Calculate JSON uses `calculate_tax` and retains the calculation report with exact holding costs.

## Tests

From the repository root, run the browser unit tests:

```bash
node --test web/*.test.mjs
```

For the WASM integration test, build a Node.js package and supply its absolute path:

```bash
wasm-pack build crates/cgt-wasm --target nodejs --dev --out-dir /tmp/cgt-display-wasm
CGT_WASM_TEST_PKG=/tmp/cgt-display-wasm node --test web/*.test.mjs
```

The WASM test job also runs this integration test.
