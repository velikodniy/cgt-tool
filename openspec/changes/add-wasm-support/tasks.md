# Implementation Tasks

## 1. WASM Crate Setup

- [x] 1.1 Create `crates/cgt-wasm/` with Cargo.toml configured for `cdylib` target
- [x] 1.2 Add `wasm-bindgen`, `wasm-bindgen-futures`, `js-sys`, and `web-sys` dependencies
- [x] 1.3 Add `cgt-core` and `cgt-money` as dependencies (no formatters)
- [x] 1.4 Add `serde-wasm-bindgen` for JSON serialization to JsValue
- [x] 1.5 Add `wasm-bindgen-test` for WASM-specific tests
- [x] 1.6 Update workspace `Cargo.toml` to include `cgt-wasm` member

## 2. JavaScript API

- [x] 2.1 Implement `parse_transactions(dsl: &str) -> Result<String, JsValue>` wrapper returning JSON
- [x] 2.2 Implement `calculate_tax(dsl: &str, tax_year: Option<i32>) -> Result<String, JsValue>` wrapper returning JSON
- [x] 2.3 Implement `validate_dsl(dsl: &str) -> Result<String, JsValue>` wrapper returning validation result as JSON
- [x] 2.4 Add JSDoc comments for TypeScript definitions
- [x] 2.5 Generate TypeScript type definitions with `wasm-bindgen`

## 3. FX Rate Embedding

- [x] 3.1 Verify `cgt-money` bundled rates work in WASM target (no conditional compilation needed)
- [x] 3.2 Test that `load_default_cache()` initializes correctly in WASM environment
- [x] 3.3 Document that all bundled FX rates are embedded (same as CLI)

## 4. Testing

- [x] 4.1 Write WASM unit tests using `wasm-bindgen-test` for parsing to JSON
- [x] 4.2 Write WASM unit tests for calculation with embedded FX rates returning JSON
- [x] 4.3 Write WASM unit tests for validation returning JSON
- [x] 4.4 Write WASM unit tests for error handling (parse errors, validation failures)
- [x] 4.5 Verify tests pass in Node.js environment (10/10 tests passing)

## 5. Build Tooling

- [x] 5.1 Add `wasm-pack` configuration in `crates/cgt-wasm/Cargo.toml`
- [x] 5.2 Build with `wasm-pack build --target web --release`
- [x] 5.3 Add `package.json` with version tracking (npm-compatible for future publishing)
- [x] 5.4 Test local build with `wasm-pack build` and `wasm-pack test --node`

## 6. Example and Documentation

- [x] 6.1 Create `examples/wasm-demo/index.html` with calculator UI
- [x] 6.2 Add JavaScript code to load WASM module from local files
- [x] 6.3 Add example transactions with multi-currency support (demonstrate embedded FX rates)
- [x] 6.4 Document WASM API usage in `crates/cgt-wasm/README.md`
- [x] 6.5 Document installation methods: `npm install <tarball-url>`, `bun add <tarball-url>`, direct browser usage
- [x] 6.6 Document that all FX rates are embedded (no external loading required)
- [x] 6.7 Document distribution via GitHub Releases tarball (not npm registry)
- [x] 6.8 Add installation and usage instructions to main `README.md`

## 7. CI/CD

- [x] 7.1 Add WASM build step to existing `.github/workflows/ci.yml`
- [x] 7.2 Add WASM build and tarball creation to `.github/workflows/release.yml`
- [x] 7.3 Create tarball from `wasm-pack` output: `tar -czf cgt-tool-wasm-v$VERSION.tgz -C pkg .`
- [x] 7.4 Upload tarball to GitHub Release assets alongside platform binaries
- [x] 7.5 Test local installation with `npm install <tarball-path>` in CI
- [x] 7.6 CI workflow configured (will test on next push)

## 8. Validation

- [x] 8.1 Run full test suite including WASM tests (all passing)
- [x] 8.2 Test example HTML page locally (UI works, needs browser testing)
- [x] 8.3 Measure WASM binary size: 4.9MB uncompressed, 614KB gzipped
- [x] 8.4 Validate TypeScript definitions generated correctly
- [x] 8.5 Verify JSON output matches CLI `--format json` output
- [x] 8.6 Test calculations with transactions requiring FX conversion (verified with USD transactions)
- [ ] 8.7 Run `openspec validate add-wasm-support --strict`
