# Change: Add WebAssembly Build Target

## Why

The CGT tool's core calculation engine is already IO-free and WASM-friendly. Compiling to WebAssembly enables browser-based CGT calculation with JSON output, providing privacy-preserving client-side computation without requiring installation. This aligns with the principle that sensitive financial data should never leave the user's machine unless explicitly chosen.

## What Changes

- Add `cgt-wasm` crate with `wasm-bindgen` bindings for core calculation and parsing functionality
- Expose JavaScript/TypeScript API for parsing DSL and calculating tax reports (JSON output only)
- Embed all bundled FX rates in WASM binary (same as CLI; compression optimization deferred)
- Add CI workflow to build WASM package on release
- Provide example HTML page demonstrating browser usage
- Keep WASM artifacts local/GitHub Releases only (npm publishing deferred)

## Impact

- **New specs**: `wasm-build` (new capability)
- **Affected code**:
  - New: `crates/cgt-wasm/` (WASM bindings for `cgt-core` only)
  - Modified: `.github/workflows/release.yml` (add WASM build step)
  - New: `examples/wasm-demo/` (browser demo)
