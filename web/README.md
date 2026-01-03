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
