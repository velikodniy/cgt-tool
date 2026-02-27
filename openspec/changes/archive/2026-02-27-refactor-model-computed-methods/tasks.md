## 1. Core model methods

- [x] 1.1 Add computed methods on `Disposal` for net gain/loss and total allowable cost.
- [x] 1.2 Add computed methods on `TaxYearSummary` for gross proceeds and taxable gain.

## 2. Formatter migration

- [x] 2.1 Replace duplicated computations in `cgt-formatter-plain` with model method calls.
- [x] 2.2 Replace duplicated computations in `cgt-formatter-pdf` with model method calls.

## 3. Verification

- [x] 3.1 Run `cargo fmt` in the worktree.
- [x] 3.2 Run `cargo clippy` in the worktree.
- [x] 3.3 Run `cargo test` in the worktree.
