# wasm-build Specification

## Purpose

TBD - created by archiving change add-wasm-support. Update Purpose after archive.

## Requirements

### Requirement: WASM Compilation

The system SHALL compile core parsing and calculation functionality to WebAssembly with JSON output.

#### Scenario: Build WASM package

- **WHEN** `wasm-pack build --target web` is run in `crates/cgt-wasm/`
- **THEN** generate WASM binary (`cgt_tool_wasm_bg.wasm`)
- **AND** generate JavaScript glue code (`cgt_tool_wasm.js`)
- **AND** generate TypeScript definitions (`cgt_tool_wasm.d.ts`)
- **AND** package only includes `cgt-core` and `cgt-money` dependencies (no formatters)

#### Scenario: WASM binary size measurement

- **WHEN** WASM package is built with `--release` flag
- **THEN** measure and report compressed WASM binary size
- **AND** binary includes embedded FX rates (expect ~1-2MB compressed)

### Requirement: JavaScript API

The system SHALL expose JavaScript functions for parsing and calculation with JSON output.

#### Scenario: Parse DSL from JavaScript

- **WHEN** `parse_transactions(dsl: string)` is called from JavaScript
- **THEN** return parsed transactions as JavaScript object (JSON structure)
- **AND** throw JavaScript Error with line numbers and context on parse failure
- **AND** JSON structure matches CLI `cgt-tool parse` output

#### Scenario: Calculate tax report

- **WHEN** `calculate_tax(dsl: string, tax_year?: number)` is called from JavaScript
- **THEN** return tax report as JavaScript object (JSON structure) with disposals, gains, and summary
- **AND** calculate for specific year if `tax_year` provided
- **AND** calculate all years if `tax_year` is null or undefined
- **AND** JSON structure matches CLI `cgt-tool report --format json` output

#### Scenario: Validate DSL

- **WHEN** `validate_dsl(dsl: string)` is called from JavaScript
- **THEN** return validation result as JavaScript object with `is_valid` boolean and array of errors
- **AND** errors include line numbers, transaction context, and suggested fixes
- **AND** JSON structure matches CLI validation output

### Requirement: TypeScript Definitions

The system SHALL provide TypeScript type definitions for all exported functions.

#### Scenario: TypeScript compilation

- **WHEN** TypeScript project imports WASM package
- **THEN** TypeScript compiler recognizes all exported functions
- **AND** provides type checking for function parameters and return types
- **AND** provides IntelliSense autocomplete in IDEs

### Requirement: Error Handling

The system SHALL convert Rust errors to JavaScript Error objects with actionable messages.

#### Scenario: Parse error conversion

- **WHEN** parsing fails in WASM
- **THEN** throw JavaScript Error with message containing line number and expected format
- **AND** error message matches CLI error format for consistency

#### Scenario: Calculation error conversion

- **WHEN** calculation fails (e.g., missing FX rate)
- **THEN** throw JavaScript Error with message describing missing data and suggested fix
- **AND** include transaction context (ticker, date, currency) when applicable

### Requirement: Browser Compatibility

The system SHALL support modern browsers with WebAssembly capability.

#### Scenario: Minimum browser versions

- **WHEN** WASM module is loaded
- **THEN** work in Chrome 57+, Firefox 52+, Safari 11+, Edge 16+
- **AND** fail with clear error message in browsers without WASM support

#### Scenario: ES module import

- **WHEN** WASM package is imported in browser
- **THEN** support ES module import syntax (`import init, { calculate_tax } from './cgt_tool_wasm.js'`)
- **AND** initialization function returns Promise that resolves when WASM is ready

### Requirement: GitHub Release Distribution

The system SHALL include WASM artifacts in GitHub Releases as tarball for local installation and self-hosting.

#### Scenario: WASM tarball in release

- **WHEN** a new version tag is released
- **THEN** GitHub Release includes tarball containing WASM package (e.g., `cgt-tool-wasm-v0.8.0.tgz`)
- **AND** tarball contains `wasm-pack` output: WASM binary, JavaScript glue, TypeScript definitions, and package.json
- **AND** tarball structure matches npm package format

#### Scenario: Local npm installation

- **WHEN** user runs `npm install <github-release-url>/cgt-tool-wasm-v0.8.0.tgz`
- **THEN** package installs to `node_modules/` with standard npm structure
- **AND** TypeScript definitions are discoverable
- **AND** imports work same as published npm packages

#### Scenario: Bun installation

- **WHEN** user runs `bun add <github-release-url>/cgt-tool-wasm-v0.8.0.tgz`
- **THEN** package installs with standard Bun structure
- **AND** imports work identically to npm

#### Scenario: Direct browser usage

- **WHEN** user extracts tarball and serves files
- **THEN** WASM module can be imported via `<script type="module">`
- **AND** example HTML page demonstrates loading from local files
- **AND** no build tools required for basic usage

### Requirement: Embedded FX Rates

The system SHALL embed all bundled FX rates for full currency conversion support.

#### Scenario: FX rates available by default

- **WHEN** WASM module is initialized
- **THEN** all bundled FX rates (2015-2025) are available immediately
- **AND** no additional loading or configuration required
- **AND** currency conversion works same as CLI

#### Scenario: Multi-currency calculations

- **WHEN** calculating tax for transactions with foreign currencies
- **THEN** use embedded HMRC rates for conversion to GBP
- **AND** handle all currencies and months covered by bundled rates
- **AND** return same results as CLI for identical input

### Requirement: Testing

The system SHALL include WASM-specific tests for browser and Node.js environments.

#### Scenario: WASM unit tests

- **WHEN** `wasm-pack test --node` is run
- **THEN** execute Rust tests compiled to WASM in Node.js environment
- **AND** tests cover parsing, calculation, and error handling

#### Scenario: Browser test environment

- **WHEN** `wasm-pack test --headless --chrome` is run
- **THEN** execute Rust tests compiled to WASM in headless Chrome
- **AND** verify browser-specific functionality (ES modules, fetch, etc.)

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
