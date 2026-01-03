# Change: Relocate WASM Demo from examples/ to web/

## Why

The WASM demo is the primary web interface for the CGT calculator, not just an "example." Moving it from `examples/wasm-demo/` to `web/` better reflects its role as a first-class project component and provides a cleaner, shorter path for users.

## What Changes

- Move `examples/wasm-demo/` contents to `web/`
- Update symlink from `web/pkg -> ../crates/cgt-wasm/pkg`
- Remove empty `examples/` directory
- Update all documentation references (README.md, AGENTS.md, cgt-wasm README, wasm-build spec)
- Update web/README.md path references and file structure

## Impact

- Affected specs: `wasm-build`
- Affected code: None (only file moves and documentation updates)
- Affected docs: `README.md`, `AGENTS.md`, `crates/cgt-wasm/README.md`, `examples/wasm-demo/README.md` (becomes `web/README.md`)
