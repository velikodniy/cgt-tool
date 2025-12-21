# WASM Build Design

## Context

The CGT tool is a Rust-based CLI that parses transaction DSL, applies HMRC tax rules, and generates reports. The core calculation engine is already IO-free, making it suitable for compilation to WebAssembly. The primary motivation is to enable privacy-preserving client-side tax calculations in web browsers without server-side processing of sensitive financial data.

**Constraints:**

- Bundled FX rates are ~4.5MB (127 XML files), will be embedded initially with compression as future optimization
- PDF generation depends on `typst-as-lib`, which may not compile to WASM
- JavaScript API should be ergonomic and provide TypeScript definitions

**Stakeholders:**

- Web developers integrating CGT calculations into financial tools
- End users wanting browser-based calculators without installation
- Privacy-conscious users preferring client-side computation

## Goals / Non-Goals

**Goals:**

- Expose core parsing and calculation to JavaScript with JSON output
- Embed all bundled FX rates (same as CLI) for full functionality
- Generate TypeScript definitions for type-safe JavaScript usage
- Maintain feature parity with CLI for core operations (parsing, calculation, validation, JSON serialization)
- Provide local WASM build that can be used directly or self-hosted

**Non-Goals:**

- Plain text or PDF formatting in WASM (formatters are separate crates; WASM only wraps `cgt-core` for JSON output)
- Full CLI feature parity (e.g., `--fx-folder` file system access is not applicable in browser)
- FX rate compression optimization (deferred to future work)
- npm publishing (deferred; keep distribution via GitHub Releases)
- Implementing a full-featured web UI (only provide example/demo)

## Decisions

### Decision: Use wasm-bindgen for JavaScript bindings

**Rationale:** `wasm-bindgen` is the standard Rust-to-WASM toolchain with excellent TypeScript definition generation, seamless integration with web_sys/js_sys, and strong ecosystem support (wasm-pack, wasm-bindgen-test).

**Alternatives considered:**

- Raw WASM exports: More manual, no automatic TypeScript definitions
- wasm-pack alone: Still uses wasm-bindgen under the hood; this is the standard approach

### Decision: Embed all FX rates initially

**Rationale:** Start with simplest approach—embed all bundled FX rates using the existing `include_dir!` mechanism that already works in `cgt-money`. This provides full functionality out of the box with no additional JavaScript code. Compression optimization (e.g., convert to binary format, use gzip, or lazy-load) can be added later if binary size becomes problematic.

**Implementation:**

- Use same FX rate loading as CLI (`cgt-money::load_default_cache()`)
- No conditional compilation needed for WASM target
- All 127 XML files embedded at compile time
- Future optimization: compress rates or lazy-load if binary size exceeds acceptable threshold

**Alternatives considered:**

- Lazy-load via JavaScript: Adds complexity, requires user to provide rates, complicates API
- Minimal bundled rates: Reduces functionality; users can't calculate older transactions
- Compress rates: Good future optimization, but adds complexity initially

### Decision: Enhanced JSON output with calculated fields

**Rationale:** The core calculation engine (`cgt-core`) produces structured data with disposals and totals, but lacks fields needed for web UIs like exemption amounts and taxable gains. The WASM layer adds a thin enhancement layer that:

- Calculates exemption amounts using `cgt-core::get_exemption()`
- Computes `total_proceeds` (sum of gross proceeds)
- Computes `total_cost` (sum of allowable costs from matches)
- Computes `taxable_gain` as `(net_gain - exemption).max(0)`
- Uses `cgt-format::format_tax_year()` for period formatting

This provides JavaScript applications with all necessary fields for rendering reports without requiring client-side calculations.

**Dependencies:** Added `cgt-format` for formatting utilities (lightweight, no I/O dependencies).

**Alternatives considered:**

- Include plain text formatter: Not needed; JavaScript can render HTML directly
- Include PDF formatter: `typst-as-lib` dependencies incompatible with WASM; out of scope
- Return raw core data only: Would require JavaScript to duplicate tax calculation logic

### Decision: Create separate `cgt-wasm` crate wrapping `cgt-core`

**Rationale:** WASM bindings require `cdylib` crate type and WASM-specific dependencies (`wasm-bindgen`, `web-sys`). A separate crate keeps the core library clean while allowing WASM-specific code (error serialization, JS interop) to coexist. The crate only depends on `cgt-core` and `cgt-money`, not formatters.

**Structure:**

```
crates/cgt-wasm/
├── Cargo.toml           # cdylib, wasm-bindgen, cgt-core, cgt-money, cgt-format
├── src/
│   ├── lib.rs           # wasm_bindgen exports, TaxYear/TaxReport structs
│   └── utils.rs         # Error conversion helpers
├── tests/
│   └── wasm.rs          # wasm-bindgen-test tests
├── package.json         # npm metadata
└── README.md            # API documentation
```

**Alternatives considered:**

- Add WASM exports to `cgt-core`: Pollutes core library with WASM-specific code
- Use feature flags: Complicates crate type switching (`lib` vs `cdylib`)
- Include formatters: Out of scope; JSON output is sufficient

### Decision: Provide interactive web demo

**Rationale:** Users need a reference implementation showing how to integrate WASM bindings. A single-file HTML demo (`examples/wasm-demo/index.html`) provides:

- Working example of WASM initialization and API usage
- Modern, responsive UI demonstrating report rendering
- Visual reference for disposal cards, match badges, and summary tables
- Testing playground for users before integration

**Design principles:**

- Single HTML file with embedded CSS/JavaScript for easy deployment
- Modern card-based layout with compact spacing
- Color-coded match badges (Same Day: blue, Bed & Breakfast: amber, Section 104: purple)
- Responsive grid layouts for different screen sizes
- Toast notifications for user feedback

**Distribution:** Demo files symlink to WASM build output (`pkg/`) for local testing.

### Decision: Local/GitHub Releases distribution only (defer npm)

**Rationale:** Start with simpler distribution via GitHub Releases. Users can download WASM artifacts and host them directly, or use them locally. This avoids npm maintenance overhead until there's proven demand. The package structure remains npm-compatible for future publishing if needed.

**Distribution format:**

- Package `wasm-pack` output (`pkg/` directory) as tarball: `cgt-tool-wasm-v0.8.0.tgz`
- Tarball works natively with `npm install <tarball>`, `bun add <tarball>`, and Deno
- Users can also extract and use files directly via `<script type="module">`
- Include tarball in GitHub Release assets alongside platform binaries

**Installation methods:**

```bash
# npm (creates node_modules/cgt-tool-wasm/)
npm install https://github.com/velikodniy/cgt-tool/releases/download/v0.8.0/cgt-tool-wasm-v0.8.0.tgz

# bun (same structure)
bun add https://github.com/velikodniy/cgt-tool/releases/download/v0.8.0/cgt-tool-wasm-v0.8.0.tgz

# deno (import directly from URL or local extraction)
import init, { calculate_tax } from './pkg/cgt_tool_wasm.js';

# Direct browser usage (extract tarball, serve files)
<script type="module">
  import init from './pkg/cgt_tool_wasm.js';
</script>
```

**Future:** Can publish to npm registry with zero code changes if demand warrants.

**Alternatives considered:**

- ZIP files: Less standard for JavaScript ecosystem; npm/bun prefer tarballs
- Individual files: Inconvenient; users must download multiple assets
- Publish to npm immediately: Adds maintenance overhead; unclear demand

## Risks / Trade-offs

### Risk: WASM binary size with embedded rates

- **Trade-off:** Embedding 4.5MB of XML increases binary size, but provides full functionality without additional JavaScript complexity
- **Mitigation:** Profile with `wasm-pack build --release` and `wasm-opt`. Strip debug symbols, enable LTO.
- **Measurement:** Check `cgt_tool_wasm_bg.wasm.gz` size after build; expect ~5-6MB uncompressed, ~1-2MB compressed
- **Future optimization:** Defer compression/lazy-loading to separate change if size becomes problematic

### Risk: TypeScript definition accuracy

- **Mitigation:** Test TypeScript definitions with example project. Use `wasm-bindgen` comments for JSDoc.
- **Validation:** Compile example TypeScript code as part of CI.

### Risk: Browser compatibility

- **Mitigation:** Test in Chrome, Firefox, Safari. Require modern browsers with WASM support (2017+).
- **Documentation:** Specify minimum browser versions in README.

## Migration Plan

Not applicable—this is a new capability, no migration needed.

**Rollout:**

1. Implement WASM crate and API
2. Test locally with example HTML page
3. Add CI workflow for WASM builds
4. Publish initial version to npm (e.g., `0.1.0-wasm.0`)
5. Gather feedback and iterate
6. Promote to stable version (`0.8.0` or `1.0.0`)

**Rollback:**

- WASM package is additive; removal does not affect CLI
- If issues arise, unpublish npm package and remove `cgt-wasm` crate

## Open Questions

1. **WASM memory management:** Should we expose manual `free()` for large objects or rely on garbage collection?

   - **Answer (to be decided):** Start with automatic GC (wasm-bindgen default); add manual free if memory issues arise.

2. **Versioning:** Should WASM artifacts track CLI version or have independent versioning?

   - **Answer (to be decided):** Match CLI version for consistency; version in lockstep.

3. **Future FX rate optimization:** When should we optimize rate loading (compression, lazy-load, binary format)?

   - **Answer (to be decided):** Wait for actual binary size measurement and user feedback; create separate change proposal if needed.

4. **Future npm publishing:** Under what conditions should we publish to npm?

   - **Answer (to be decided):** If users request easier integration or if we want wider adoption; requires minimal changes (already npm-compatible).
