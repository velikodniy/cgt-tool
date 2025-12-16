# Design

## Scope

Publish `cgt-tool` through the existing `velikodniy/homebrew-tap`, using prebuilt GitHub release binaries and keeping the formula current via automation. No changes to the Rust workspace are required; work is split between the tap (formula + workflow) and the main repo (docs).

## Formula layout

- Use upstream release assets directly to avoid slow source builds: `cgt-tool-macos-aarch64`, `cgt-tool-macos-x86_64`, `cgt-tool-linux-aarch64`, `cgt-tool-linux-x86_64` from tag `vX.Y.Z`.
- Structure with `on_macos` and `on_linux` blocks, each specifying `url`, `sha256`, and `on_intel`/`on_arm` to map to the correct binary.
- Install by renaming the downloaded asset to `cgt-tool` (dropping extension) via `bin.install`.
- `test do` runs `cgt-tool --version` to confirm the binary links on the host.

## Checksum source

- Prefer the published `checksums.txt` asset for versioned SHA256 values; fall back to computing checksums after downloading each asset if needed.
- Keep SHA updates deterministic so automation can rewrite the formula without manual edits.

## Automation approach

- Add a workflow in the tap repo (no OpenSpec tooling there) that:
  - Triggers on a schedule and `workflow_dispatch`; optionally accept a target tag input.
  - Fetches the latest `velikodniy/cgt-tool` release via GitHub API.
  - Compares the tag to the current `VERSION` in `Formula/cgt-tool.rb` to decide whether to update.
  - Downloads `checksums.txt` (or assets) to populate per-arch SHA256s and rewrites the formula URLs/version/sha fields.
  - Runs a smoke install (`brew install velikodniy/tap/cgt-tool && cgt-tool --version`) on macOS; Linux run is optional but planned when runners are available.
  - Commits and pushes the formula change (or opens a PR) with the updated version.

## Documentation updates

- cgt-tool README gains a Homebrew installation section showing `brew tap velikodniy/tap` and `brew install cgt-tool`.
- Tap README lists `cgt-tool` alongside existing formulas.

## Open questions

- Whether to keep Linux binaries in the formula initially; scope assumes yes since release assets exist, but this can be limited to macOS if audit tooling requires it.
- Whether automation should open PRs vs direct pushes; default to direct push (matching `update-openspec`) unless reviewer sign-off is required.
