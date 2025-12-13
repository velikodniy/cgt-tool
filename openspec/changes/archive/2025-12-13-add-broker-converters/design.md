# Design: Broker Export Converters

## Context

The CGT calculator uses a custom DSL for transaction input. Users with brokerage accounts must manually convert their export files (CSV/JSON) to this format. This is error-prone and prevents adoption. We need an extensible converter system that:

1. Supports multiple broker formats
2. Is WASM-compatible for browser-based tools
3. Produces well-documented `.cgt` output with metadata

**Constraints:**

- Must be IO-free at the core (string-in, string-out) for WASM compatibility
- Must handle broker-specific quirks (date formats, column names, action types)
- Must support multi-file inputs (e.g., Schwab transactions CSV + awards JSON)

## Goals / Non-Goals

**Goals:**

- Extensible trait-based design for adding new brokers
- Complete Schwab support (transactions CSV + equity awards JSON)
- WASM-friendly: no filesystem/network IO in converter core
- Generated comments with source metadata and warnings
- CLI integration via `cgt convert` subcommand

**Non-Goals:**

- Automatic broker detection (user specifies broker)
- Real-time API integration with brokerages
- Support for all Schwab transaction types (focus on CGT-relevant: Buy, Sell, Split, Dividend, Stock Activity)

## Decisions

### Decision 1: Single Crate with Modules (not separate crates per broker)

**Rationale:** Broker converters share significant infrastructure (CSV parsing, date handling, currency formatting). Separate crates would create:

- Excessive boilerplate for small modules (~200-500 lines each)
- Complex workspace dependencies
- Poor discoverability

**Structure:**

```
crates/cgt-converter/
├── src/
│   ├── lib.rs           # Public API, Converter trait
│   ├── error.rs         # Converter errors
│   ├── output.rs        # CGT file generation utilities
│   ├── schwab/          # Schwab module
│   │   ├── mod.rs
│   │   ├── transactions.rs
│   │   └── awards.rs
│   ├── trading212/      # Future: Trading 212 module
│   └── degiro/          # Future: Degiro module
└── tests/
    ├── schwab_tests.rs
    └── fixtures/
        └── schwab/
```

### Decision 2: String-Based API with Broker-Specific Input Types (WASM-friendly)

**Rationale:** WASM environments cannot access the filesystem. The converter must accept raw file contents as strings. Different brokers have different input requirements (Schwab needs transactions CSV + optional awards JSON, Trading 212 needs one or more CSVs, etc.).

**API:**

```rust
pub trait BrokerConverter {
    /// Broker-specific input type
    type Input;

    /// Convert broker export(s) to CGT DSL format
    fn convert(&self, input: &Self::Input) -> Result<ConvertOutput, ConvertError>;

    /// Broker identifier for CLI/logging
    fn broker_name(&self) -> &'static str;
}

/// Schwab-specific input
pub struct SchwabInput {
    pub transactions_csv: String,
    pub awards_json: Option<String>,
}

/// Trading 212-specific input (future)
pub struct Trading212Input {
    pub files: Vec<String>,
}

/// Output with metadata for comment generation
pub struct ConvertOutput {
    pub cgt_content: String,
    pub warnings: Vec<String>,
    pub skipped_count: usize,
}
```

This design allows each broker to define its exact input requirements while maintaining a consistent conversion interface.

### Decision 3: RSU Vesting = BUY at Fair Market Value

**Rationale:** Per HMRC rules, when RSUs vest:

- Shares are acquired at their Fair Market Value (FMV) on vesting date
- This FMV becomes the cost basis for CGT calculations
- Some shares may be immediately sold for tax withholding (separate SELL transaction)

Schwab "Stock Plan Activity" with FMV from awards file → BUY transaction in CGT DSL.

### Decision 4: Generated Output Format

Output includes:

1. **Header comments** with metadata (source broker, conversion date, file names)
2. **Transaction comments** for notable items (e.g., RSU vesting, unsupported actions skipped)
3. **Chronological ordering** (oldest first, as expected by CGT calculator)

**Example output:**

```
# Converted from Charles Schwab export
# Source files: transactions.csv, awards.json
# Converted: 2025-01-15T10:30:00Z
# WARNING: 3 unsupported transactions skipped (Wire Sent, Credit Interest)

# RSU Vesting - FMV from awards file
2023-04-25 BUY GOOG 67.2 @ 125.6445 USD

# Regular purchase
2023-05-10 BUY GOOG 10 @ 130.00 USD EXPENSES 4.95 USD

# Sale (partial - tax withholding)
2023-06-14 SELL GOOG 62.601495 @ 113.75 USD EXPENSES 0.17 USD
```

### Decision 5: Schwab Transaction Type Mapping

| Schwab Action                             | CGT DSL  | Notes                                             |
| ----------------------------------------- | -------- | ------------------------------------------------- |
| Buy                                       | BUY      | Direct mapping                                    |
| Sell                                      | SELL     | Direct mapping                                    |
| Stock Plan Activity                       | BUY      | At FMV from awards file                           |
| Stock Split                               | SPLIT    | Requires ratio calculation                        |
| Cash Dividend                             | DIVIDEND | Optional support                                  |
| Qualified Dividend                        | DIVIDEND | Optional support                                  |
| Short/Long Term Cap Gain                  | DIVIDEND | Distribution from funds                           |
| NRA Tax Adj / NRA Withholding             | —        | Tax on dividends (included in DIVIDEND TAX field) |
| Wire Sent/Received, Credit Interest, etc. | —        | Skipped (not CGT-relevant)                        |

## Risks / Trade-offs

| Risk                           | Mitigation                                                                                          |
| ------------------------------ | --------------------------------------------------------------------------------------------------- |
| Schwab format changes          | Version detection, clear error messages pointing to format mismatch                                 |
| Missing FMV for RSU vesting    | Error if awards file needed but not provided; skip with warning if Stock Plan Activity has no price |
| Incorrect transaction matching | Comprehensive test fixtures from real-world exports                                                 |
| WASM bundle size               | Feature flags if needed, but expect minimal impact (~50KB)                                          |

## Migration Plan

No migration needed — this is a new capability. Existing `.cgt` files are unaffected.

**Rollout:**

1. Implement Schwab converter with full test coverage
2. Add CLI `convert` subcommand
3. Update documentation with usage examples
4. Future: Add Trading 212, Degiro as separate modules

## Open Questions

1. **Dividend handling**: Should we include dividends in the converter output? They're not strictly required for CGT but useful for record-keeping. The DSL already supports DIVIDEND.

   - **Proposed**: Yes, include dividends. Users can remove if unwanted.

2. **Stock split ratio**: Schwab doesn't directly provide split ratios in transactions. Need to infer from quantity changes or require manual input.

   - **Proposed**: For initial release, output a comment requiring manual ratio entry. Future: lookup table for common splits.
