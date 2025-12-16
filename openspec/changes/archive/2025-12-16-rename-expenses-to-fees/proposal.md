# Rename EXPENSES to FEES

## Background

The current DSL uses the keyword `EXPENSES` to denote transaction costs. The user has requested to rename this to `FEES` as it is more concise and arguably more natural for trading contexts ("transaction fees"). The user also inquired about the usage of `@` for price, which is retained as it is standard and intuitive.

## Goal

Rename the `EXPENSES` keyword to `FEES` in the DSL grammar, parser, and all related documentation and tests.

## Scope

- **DSL Grammar**: Update `parser.pest` to use `FEES` instead of `EXPENSES`.
- **Parsing Logic**: Update AST mapping if necessary.
- **Broker Conversion**: Update `cgt-converter` to output `FEES`.
- **Tests**: Update all `.cgt` input files and expected output files (`.json`, `.txt`) to reflect the change.
- **Documentation**: Update specs and examples.

## Risks

- **Breaking Change**: This is a breaking change for the DSL. Existing `.cgt` files will fail to parse unless updated. Since this is a CLI tool, users will need to update their input files.
- **Test churn**: Many tests use `EXPENSES` and will need updating.

## Alternatives Considered

- **Support both**: We could support both `EXPENSES` and `FEES` for backward compatibility, but this adds complexity to the grammar and we prefer a single canonical way. The user request implies a replacement.
