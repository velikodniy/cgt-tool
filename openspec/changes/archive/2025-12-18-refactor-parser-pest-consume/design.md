# Design: Parser Refactor with pest_consume

## Context

The current parser uses `pest` directly with ~400 lines of manual parsing code and ~90 lines of custom error handling. The grammar uses intermediate wrapper rules and separate clause rules that obscure the semantic structure of the DSL. `pest_consume` is a higher-level parser library built on top of `pest` that provides:

- Derive-based parsing with semantic methods per grammar rule
- Automatic error type conversions
- Cleaner separation between grammar (syntax) and parsing logic (semantics)
- Type-safe parser combinators that match Rust's type system

This refactor aims to simplify the codebase while maintaining full backward compatibility with existing DSL files.

## Goals / Non-Goals

**Goals:**

- Simplify parser implementation using `pest_consume`
- Make grammar more semantic and self-documenting
- Reduce manual error handling code
- Maintain 100% backward compatibility with existing DSL syntax
- Preserve all existing parser tests and behavior

**Non-Goals:**

- Change DSL syntax visible to users
- Modify transaction semantics or validation rules
- Change error message format or quality (should improve, but not a primary goal)
- Add new DSL features (pure refactor)

## Decisions

### Decision 1: Use pest_consume instead of raw pest

**Rationale:**

- `pest_consume` provides derive-based parsing that reduces boilerplate
- Type-safe semantic methods match grammar rules 1:1
- Automatic error handling reduces custom error conversion code
- Well-maintained library with good Rust ecosystem integration

**Alternatives considered:**

- Continue with raw `pest`: Rejected due to high manual parsing overhead
- Switch to different parser (nom, chumsky): Rejected due to grammar rewrite cost and unfamiliarity
- Write hand-rolled recursive descent parser: Rejected due to loss of PEG grammar benefits

### Decision 2: Restructure grammar to use semantic types

**Rationale:**

- Embedding markers in semantic types (e.g., `price = { "@" ~ money }`) makes grammar self-documenting
- Eliminates intermediate wrapper rules (`buy_sell_args`, etc.) that add indirection without semantic value
- Flattens command rules for clearer structure
- Makes the grammar more closely match the domain model

**Example:**

```
// Before (manual clauses)
buy_sell_args = { ticker ~ quantity ~ "@" ~ money ~ fees_clause? }
fees_clause = { ^"FEES" ~ money }

// After (semantic types)
cmd_buy = { ^"BUY" ~ ticker ~ quantity ~ price ~ fees? }
price = { "@" ~ money }
fees = { ^"FEES" ~ money }
```

**Alternatives considered:**

- Keep existing grammar structure: Rejected because it doesn't leverage semantic types well
- More radical grammar changes: Rejected to minimize migration risk

### Decision 3: Make atomic rules explicit with @ prefix

**Rationale:**

- Making `date` and other base types atomic (using `@`) tells pest to not create spans for inner rules
- This aligns with pest best practices and improves performance
- Makes it clear which rules produce single tokens vs composite structures

### Decision 4: Remove manual error helpers in favor of pest_consume errors

**Rationale:**

- `pest_consume` provides its own error type with rich context
- Current manual helpers (~90 lines) duplicate functionality already in pest_consume
- Error conversion can be implemented in `CgtError::From<pest_consume::Error>`
- May need to enhance error messages in the conversion, but basic structure is provided

**Trade-offs:**

- We lose some custom error message formatting from `format_rule_name`
- We gain automatic error propagation and simpler code
- Error quality should remain similar or improve with pest_consume's built-in context

### Decision 5: Implement semantic methods per grammar rule

**Rationale:**

- Each grammar rule gets a corresponding method on the parser struct
- Methods return properly typed Rust values (Transaction, CurrencyAmount, etc.)
- Follows pest_consume patterns and best practices
- Type system enforces correct parsing logic

**Pattern:**

```rust
type ParseResult<T> = std::result::Result<T, pest_consume::Error<Rule>>;
type Node<'i> = pest_consume::Node<'i, Rule, ()>;

#[pest_consume::parser]
impl CgtParser {
    fn transaction(input: Node) -> ParseResult<Transaction> {
        // Parse transaction and return structured data
    }

    fn money(input: Node) -> ParseResult<CurrencyAmount> {
        // Parse money amount with optional currency
    }
}
```

## Risks / Trade-offs

### Risk: pest_consume error messages differ from current format

**Mitigation:**

- Implement custom `From<pest_consume::Error>` for `CgtError` that formats messages consistently
- Test error cases explicitly to verify quality
- Can enhance error messages in conversion layer if needed

### Risk: Grammar changes break existing DSL files

**Mitigation:**

- Grammar changes are semantic-only; syntax remains identical
- All existing test fixtures must pass
- Run full integration test suite before/after
- Document any subtle behavior changes in proposal

### Risk: pest_consume introduces new dependency with maintenance concerns

**Mitigation:**

- `pest_consume` is actively maintained and widely used
- Built on top of `pest` which we already depend on
- Small, focused library with stable API
- Version 2.x series is mature

### Trade-off: More code in parser.rs vs fewer helpers

The refactor replaces ~90 lines of error helpers with semantic methods (~40-50 lines per major rule type). Net result should be similar or slightly fewer lines, but more structured and maintainable.

## Migration Plan

1. **Add dependency**: Update `Cargo.toml` with `pest_consume = "2.7"`
2. **Update grammar**: Restructure `parser.pest` following semantic type pattern
3. **Verify grammar**: Test that grammar still parses with `pest` before converting parser code
4. **Implement pest_consume parser**: Replace parser.rs implementation incrementally
5. **Add error conversion**: Implement `From<pest_consume::Error>` for `CgtError`
6. **Test**: Run full test suite, verify all parser_tests.rs pass
7. **Integration test**: Run all fixture files through refactored parser
8. **Update docs**: Update MCP resources if needed

**Rollback strategy:**

- Keep changes in feature branch until fully tested
- Grammar file can be reverted independently if needed
- Parser implementation is isolated to one module

## Open Questions

None - design is straightforward and low-risk. Implementation follows established pest_consume patterns.
