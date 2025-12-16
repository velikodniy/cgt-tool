# Change: Publish cgt-tool via Homebrew Tap

## Why

- Issue #5 requests easy installation through the existing `velikodniy/homebrew-tap` instead of manual binary downloads.
- Homebrew formulae need explicit checksums per platform; keeping them in a tap improves discoverability and updates.
- Automating formula refreshes prevents stale versions and reduces manual checksum maintenance.

## What Changes

- Add a `Formula/cgt-tool.rb` in `velikodniy/homebrew-tap` that downloads release binaries for macOS (arm64/x86_64) and Linux (arm64/x86_64), with matching SHA256 per asset and a `--version` smoke test.
- Document Homebrew installation in the cgt-tool README (and list it in the tap README) using `brew tap velikodniy/tap` and `brew install cgt-tool`.
- Add GitHub Actions automation to update the tap formula when new cgt-tool releases publish, reusing release checksums and committing the version bump, plus an on-demand trigger.
- Validate installation on macOS runners (and Linux where applicable) as part of the automation to ensure the binary runs after formula updates.

## Impact

- Affected specs: updates under `ci` for tap distribution and automation.
- Affected files/repos:
  - `velikodniy/homebrew-tap`: `Formula/cgt-tool.rb` (new), README entry, workflow for formula updates/tests.
  - `velikodniy/cgt-tool`: README installation section for Homebrew.
- No changes to the Rust codebase or release artifacts are expected.

## Design Considerations

- Use release asset names (`cgt-tool-macos-aarch64`, `cgt-tool-macos-x86_64`, `cgt-tool-linux-aarch64`, `cgt-tool-linux-x86_64`) with explicit URLs and checksums rather than building from source inside the formula to keep install fast.
- Prefer `brew test` to call `cgt-tool --version` for a quick sanity check without needing sample data.
- Formula update workflow should avoid duplicating OpenSpec tooling and run entirely within the tap repo, triggered by cgt-tool releases or manual dispatch.
- If future releases add bottles or universal binaries, keep the formula structure segmented (`on_macos` / `on_linux`) to swap sources without changing consumers.
