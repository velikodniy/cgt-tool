# Tasks: Rename EXPENSES to FEES

- [x] Update DSL grammar (`parser.pest`) to use `FEES` instead of `EXPENSES` <!-- id: grammar -->
- [x] Update `dsl-parsing` spec to require `FEES` <!-- id: spec-dsl -->
- [x] Update `broker-conversion` spec to output `FEES` <!-- id: spec-broker -->
- [x] Update `cgt-converter` to output `FEES` in generated DSL <!-- id: converter -->
- [x] Update `cgt-formatter-plain` if it outputs DSL strings (it seems to output human readable strings with "fees" already, but check for any DSL generation) <!-- id: formatter -->
- [x] Update all `.cgt` test files in `tests/inputs/` <!-- id: test-inputs -->
- [x] Update all expected JSON files in `tests/json/` <!-- id: test-json -->
- [x] Update all plain text expected outputs in `tests/plain/` (if they echo the DSL) <!-- id: test-plain -->
- [x] Update integration tests in `crates/cgt-core/tests/` and `crates/cgt-converter/tests/` <!-- id: integration-tests -->
- [x] Verify `cargo test` passes <!-- id: verify -->
