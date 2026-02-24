## Context

`cgt-core` defines two structurally identical types (`ValidationError`, `ValidationWarning`) with the same four fields and near-identical `Display` impls. These are public types re-exported from `cgt-core::lib.rs` and consumed by `cgt-wasm`. No other crates (CLI, MCP, formatters) reference them directly.

## Goals / Non-Goals

**Goals:**

- Eliminate structural duplication by merging the two types into one
- Preserve identical `Display` output and validation semantics
- Keep `ValidationResult` API ergonomic for consumers that distinguish errors from warnings

**Non-Goals:**

- Changing validation logic or adding new validation checks
- Merging the `errors` and `warnings` vectors in `ValidationResult` into a single list (keeping separate vectors is cleaner for consumers)

## Decisions

### 1. Introduce `Severity` enum and `ValidationIssue` struct

Replace `ValidationError` and `ValidationWarning` with:

```rust
enum Severity { Error, Warning }
struct ValidationIssue { severity, line, date, ticker, message }
```

**Rationale**: Direct mechanical replacement. The severity field captures the only difference between the two types.

**Alternative considered**: A single `Vec<ValidationIssue>` in `ValidationResult` with filtering by severity. Rejected because it forces every consumer to filter, and the existing two-vector pattern (`errors`, `warnings`) is clearer and already well-tested.

### 2. Keep separate `errors` and `warnings` fields in `ValidationResult`

Both fields become `Vec<ValidationIssue>`. Construction code sets the appropriate `Severity` variant.

**Rationale**: Minimizes churn in consumers. Tests already access `result.errors` and `result.warnings` directly.

### 3. Update WASM JSON bridge with unified conversion

The WASM crate's `ValidationErrorJson` and `ValidationWarningJson` are also structurally identical. Unify them into `ValidationIssueJson` with a single `From<&ValidationIssue>` impl.

## Risks / Trade-offs

- **[Breaking public API]** → Acceptable for a pre-1.0 crate. The old type names are removed. Consumers must update imports. Mitigated by the fact that only `cgt-wasm` is an external consumer.
- **[Severity mismatch possible]** → A `ValidationIssue` with `Severity::Warning` could theoretically be pushed to the `errors` vec. This is an internal-only concern; the `validate()` function is the sole producer. No mitigation needed beyond code review.
