## 1. Shared Ordering Utility

- [x] 1.1 Add a canonical date+ticker comparator helper in `cgt-core` with minimal public API surface.
- [x] 1.2 Replace calculator-local date+ticker comparator usage with the shared helper.

## 2. Formatter Integration

- [x] 2.1 Replace plain formatter date+ticker comparator usage with the shared helper.
- [x] 2.2 Replace PDF formatter date+ticker comparator usage with the shared helper.

## 3. Verification

- [x] 3.1 Run `cargo fmt` and ensure formatting-only diffs are clean.
- [x] 3.2 Run `cargo clippy` and resolve all warnings/errors without suppression attributes.
- [x] 3.3 Run `cargo test` and confirm no ordering regressions in formatter outputs.
