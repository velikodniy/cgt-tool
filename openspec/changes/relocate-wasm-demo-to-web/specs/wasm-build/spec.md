## MODIFIED Requirements

### Requirement: Documentation

The system SHALL provide clear documentation for JavaScript developers.

#### Scenario: API documentation

- **WHEN** developer reads `crates/cgt-wasm/README.md`
- **THEN** documentation explains installation via tarball URL with npm/bun
- **AND** includes code examples for common use cases (parse, calculate, validate)
- **AND** documents distribution via GitHub Releases tarball (not npm registry)
- **AND** shows direct browser usage by extracting tarball
- **AND** documents that all FX rates are embedded (no external loading needed)
- **AND** clarifies that WASM output is JSON only (no plain text or PDF formatting)

#### Scenario: Example HTML page

- **WHEN** developer views `web/index.html`
- **THEN** example demonstrates loading WASM module, parsing DSL, calculating tax, and displaying JSON results
- **AND** includes multi-currency transactions to demonstrate embedded FX rates
- **AND** shows how to render JSON output in browser UI
- **AND** example runs in browser without build tools (ES modules via CDN)
