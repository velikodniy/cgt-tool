## Context

Currently `crates/cgt-cli/src/main.rs` handles output inside the `match format` block.

- `Plain` and `Json` print directly to stdout, ignoring `--output`.
- `Pdf` logic handles file writing and default filenames.

## Goals

- Support `--output` for Plain and JSON formats.
- Maintain existing behavior for PDF (defaults to file).
- Keep stdout default for Plain/JSON.

## Decisions

### Decision 1: Refactor Output Logic

We will modify the `match format` block in `main.rs`.
For `Plain` and `Json` cases:

1. Generate the report string.
2. Check if `output` path is provided.
3. If provided: Write string to file.
4. If not provided: Print to stdout.

For `Pdf`:

- Keep existing logic (it already handles defaults and file writing).
