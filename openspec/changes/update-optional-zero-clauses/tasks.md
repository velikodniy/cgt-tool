## 1. Grammar Update

- [x] 1.1 Update `parser.pest` to make TAX clause optional in dividend_args
- [x] 1.2 Add `tax_clause` rule similar to existing `fees_clause`

## 2. Parsing Logic

- [x] 2.1 Update dividend parsing to default TAX to 0 when omitted
- [x] 2.2 Ensure existing tests still pass (TAX 0 syntax remains valid)

## 3. Testing

- [x] 3.1 Add test case for dividend without TAX clause
- [x] 3.2 Add test case for BUY/SELL without FEES clause (if not covered)
- [x] 3.3 Run full test suite to verify backward compatibility

## 4. Documentation

- [x] 4.1 Update any user-facing documentation to reflect optional clauses

## 5. MCP Server Documentation

- [x] 5.1 Update DSL_SYNTAX resource in resources.rs (make TAX optional in syntax and examples)
- [x] 5.2 Update DSL_SYNTAX_REFERENCE constant in server.rs
- [x] 5.3 Update convert_to_dsl tool description in server.rs
- [x] 5.4 Update convert_to_dsl implementation to omit TAX 0 when tax is zero

## 6. AGENTS.md

- [x] 6.1 Add reminder about updating MCP tool descriptions when modifying DSL syntax
