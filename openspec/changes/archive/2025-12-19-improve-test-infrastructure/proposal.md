# Change: Improve Test Infrastructure

## Why

The current test structure embeds inline test modules within source files, increasing file length and mixing concerns. Test coverage is unmeasured, making it impossible to identify untested code paths. Additionally, edge cases from HMRC guidance and real-world scenarios (multi-currency, same-day sells, complex multi-year transactions) are underrepresented.

## What Changes

- Move inline `#[cfg(test)]` modules from source files to separate `tests/` directories for each crate
- Add test coverage measurement using `cargo-llvm-cov` (modern LLVM-based coverage)
- Create cross-validation scripts that compare results against:
  - [KapJI/capital-gains-calculator](https://github.com/KapJI/capital-gains-calculator) (Python, via `uvx`)
  - [mattjgalloway/cgtcalc](https://github.com/mattjgalloway/cgtcalc) (Swift, build from source)
- Add comprehensive edge case tests derived from HMRC forums and guidance
- Create a realistic multi-year (2-3 years) complex transaction fixture

## Impact

- Affected specs: testing (new capability)
- Affected code:
  - `crates/cgt-core/src/*.rs` (remove inline tests)
  - `crates/cgt-core/tests/` (add new test files)
  - `crates/cgt-money/src/*.rs` (remove inline tests)
  - `crates/cgt-money/tests/` (add new test files)
  - `crates/cgt-formatter-*/src/*.rs` (remove inline tests)
  - `crates/cgt-converter/src/*.rs` (remove inline tests)
  - `crates/cgt-mcp/src/*.rs` (remove inline tests)
  - `scripts/` (new cross-validation scripts)
  - `.github/workflows/ci.yml` (coverage reporting)
  - `tests/inputs/` (new complex fixtures)
