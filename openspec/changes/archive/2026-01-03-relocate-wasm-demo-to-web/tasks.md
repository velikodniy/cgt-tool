## 1. File Relocation

- [x] 1.1 Create `web/` directory at repository root
- [x] 1.2 Move `examples/wasm-demo/index.html` to `web/index.html`
- [x] 1.3 Move `examples/wasm-demo/styles.css` to `web/styles.css`
- [x] 1.4 Move `examples/wasm-demo/app.js` to `web/app.js`
- [x] 1.5 Move `examples/wasm-demo/README.md` to `web/README.md`
- [x] 1.6 Remove old symlink `examples/wasm-demo/pkg`
- [x] 1.7 Create new symlink `web/pkg -> ../crates/cgt-wasm/pkg`
- [x] 1.8 Remove empty `examples/wasm-demo/` directory
- [x] 1.9 Remove empty `examples/` directory

## 2. Documentation Updates

- [x] 2.1 Update root `README.md`: Change `examples/wasm-demo/` references to `web/`
- [x] 2.2 Update root `AGENTS.md`: Add `web/` to the Structure section
- [x] 2.3 Update `crates/cgt-wasm/README.md`: Change `examples/wasm-demo/` references to `web/`
- [x] 2.4 Update `web/README.md`: Update file structure section from `wasm-demo/` to `web/`

## 3. Spec Updates

- [x] 3.1 Update `openspec/specs/wasm-build/spec.md`: Change `examples/wasm-demo/` to `web/` in documentation scenario

## 4. Validation

- [x] 4.1 Verify symlink resolves correctly: `ls -la web/pkg`
- [x] 4.2 Test local server: `python3 -m http.server 8000` from `web/` directory
- [x] 4.3 Run `openspec validate --strict` to confirm spec changes are valid
