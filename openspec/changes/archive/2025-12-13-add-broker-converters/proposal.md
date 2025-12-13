# Change: Add Broker Export Converters

## Why

Users need to import transaction history from brokerage platforms (Charles Schwab, Trading 212, Degiro, etc.) into the CGT calculator. Currently, users must manually transcribe data into the custom DSL format, which is error-prone and time-consuming. An extensible converter system will automate this process and enable WASM-based web tools.

## What Changes

- **NEW**: `cgt-converter` crate — WASM-friendly library with pluggable broker modules
- **NEW**: Schwab converter module supporting:
  - Regular transactions CSV (Buy, Sell, Dividend, Stock Split, etc.)
  - Equity Awards CSV (for RSU vesting fair market value prices)
- **NEW**: Converter trait system for future broker support (Trading 212, Degiro)
- **UPDATE**: CLI gains `convert` subcommand to transform broker exports to `.cgt` files
- **UPDATE**: Documentation with converter usage examples

## Impact

- Affected specs: NEW `broker-conversion` capability
- Affected code:
  - `crates/cgt-converter/` — new crate
  - `crates/cgt-cli/src/commands.rs` — new convert subcommand
  - `README.md` — usage documentation
