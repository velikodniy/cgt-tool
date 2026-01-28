## Context

The current Schwab converter uses a "bag of options" struct (`SchwabTransaction`) where all fields are optional strings. The parsing logic manually checks the "Action" string and then attempts to unwrap fields like "Quantity" or "Price". This approach has several issues:

1. **Runtime Fragility**: If a transaction type is encountered that doesn't match the parser's implicit assumptions (e.g., expected to be ignored or have certain fields), it can cause runtime errors or require ad-hoc fixups.
2. **Weak Typing**: The domain model doesn't reflect the actual data structure. A "Buy" transaction *always* has a quantity and price, but the struct says `Option<Decimal>`.
3. **Noise**: The parser emits warnings for every unknown transaction, even irrelevant ones, cluttering the output.

## Goals / Non-Goals

**Goals:**

- **Type Safety**: Enforce valid transaction states at the deserialization boundary using Rust's type system.
- **Resilience**: Gracefully handle unknown or irrelevant transaction types without crashing or manual string matching.
- **Clean Output**: Eliminate warnings for expectedly ignored transactions, capturing them as comments instead.

**Non-Goals:**

- Supporting every possible Schwab transaction type immediately (we focus on Buy, Sell, Stock Plan Activity, Dividends).
- Changing the output format (the generated `.cgt` file content should remain functionally identical, modulo comments).

## Decisions

### 1. Polymorphic Deserialization with `serde`

We will use `serde`'s tagged enum support to handle polymorphism.

**Decision**: Use `#[serde(tag = "Action")]` to dispatch to specific struct variants for **known, required** transactions (Buy, Sell, etc.).
**Rationale**: This pushes validation to the parsing layer. If a "Buy" record is missing "Quantity", deserialization fails early and specifically.

**Alternatives Considered**:

- *Explicitly handling a specific irrelevant transaction*: We considered adding dedicated variants for ignored actions, but decided against it. Irrelevant transactions should just fall into the "Unknown" bucket and be ignored. This is cleaner and requires less maintenance.

### 2. Fallback for Unknown Types

We will use an untagged enum wrapper to capture unknown types.

```rust
#[derive(Deserialize)]
#[serde(untagged)]
enum SchwabRecord {
    Known(Transaction),
    Unknown(RawTransaction),
}
```

**Rationale**: This ensures that if we encounter a new or weird transaction type, we don't fail the entire file. We catch it as `Unknown`, allowing us to log it or write a comment to the output file, preserving the "safe by default" principle.

### 3. Strict vs. Loose Typing

We will use custom deserializers (`deserialize_with`) to parse strings like `"$1,234.56"` directly into `Decimal` within the struct fields.

**Rationale**: Keeps the struct clean (`quantity: Decimal` instead of `quantity_str: Option<String>`).

## Risks / Trade-offs

### Risk: Schema Evolution

If Schwab changes a field name (e.g., "Quantity" -> "Qty") for a "Buy" transaction, our strict parser will reject it.
**Mitigation**:

- The fallback `Unknown` variant will catch these failures, so the tool won't crash. We will see "Skipped unknown transaction" in the output, prompting an investigation.
- Unit tests with real data samples (like the ones in `examples/`) ensure we cover the current format.

### Risk: Complexity

The struct definitions will be more verbose than the single flat struct.
**Trade-off**: Accepted. The verbosity buys us safety and clarity. The complexity is declarative (struct definitions) rather than imperative (nested `if/else` blocks).
