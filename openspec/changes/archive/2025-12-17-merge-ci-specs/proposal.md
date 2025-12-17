# Change: Merge ci and ci-cd specs

## Why

Having both `ci` and `ci-cd` specs creates confusion - both cover CI/CD automation but from different angles. The `ci` spec covers Homebrew tap distribution (tap-side), while `ci-cd` covers this repository's workflows. Consolidating into one spec simplifies maintenance and provides a single source of truth for all CI/CD concerns.

## What Changes

- Merge the `ci` spec content into `ci-cd`
- Delete the `ci` spec
- Update `ci-cd` purpose to cover both cgt-tool workflows and tap distribution requirements

## Impact

- Affected specs: `ci-cd` (merge content from `ci`), `ci` (delete)
- No code changes required - this is spec consolidation only
