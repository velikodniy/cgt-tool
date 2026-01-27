## Why

The CLI currently ignores the `--output` argument when generating Plain Text or JSON reports, printing to stdout instead. This confuses users who explicitly provide an output file path and expect the report to be saved there.

## What Changes

Update the report generation logic in `crates/cgt-cli/src/main.rs` to write to the specified file (if provided) for all output formats, not just PDF.

## Capabilities

### Modified Capabilities

- `cli-report`: Respects `--output` flag for all formats.

## Impact

- `crates/cgt-cli/src/main.rs`: Refactor output handling logic.
