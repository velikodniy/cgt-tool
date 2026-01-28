## 1. Preparation

- [x] 1.1 Read existing Schwab converter code to identify all currently supported transaction types.
- [x] 1.2 Create a reproduction test case with an unknown transaction type to confirm current failure behavior (or unsafe handling).

## 2. Implementation

- [x] 2.1 Refactor `crates/cgt-converter/src/schwab/mod.rs`: Define the polymorphic `Transaction` enum with variants for `Buy`, `Sell`, `Stock Plan Activity`, `Cash Dividend`, etc.
- [x] 2.2 Define the fallback `SchwabRecord` enum (Known/Unknown) and custom deserializers for Decimal fields.
- [x] 2.3 Update `SchwabConverter::convert` to use the new parsing logic and handle the `Transaction` enum variants.
- [x] 2.4 Update the reporting logic to handle unknown transactions as comments, without console warnings.

## 3. Verification

- [x] 3.1 Run the reproduction test case to verify unknown transactions are treated as ignored without crashing.
- [x] 3.2 Run existing tests to ensure no regression in `Buy`/`Sell`/`Dividend` parsing.
- [x] 3.3 Verify with real data that output is correct.
