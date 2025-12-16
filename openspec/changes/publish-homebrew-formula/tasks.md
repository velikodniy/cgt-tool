# Tasks

## 1. Homebrew Formula

- [x] 1.1 Create `Formula/cgt-tool.rb` with desc/homepage/version and macOS+Linux arches using release asset URLs
- [x] 1.2 Add per-arch SHA256 from GitHub release checksums and set binary name in `bin.install`
- [x] 1.3 Add `test do` block running `cgt-tool --version`
- [x] 1.4 Run `brew audit --strict --tap velikodniy/tap cgt-tool` and fix issues
- [x] 1.5 Run `brew install velikodniy/tap/cgt-tool` on macOS (arm64); Linux install not run here (no runner)

## 2. Documentation

- [x] 2.1 Update `velikodniy/cgt-tool` README with Homebrew installation steps (`tap` + `install`)
- [x] 2.2 Update tap README to list the new `cgt-tool` formula with usage snippet (now in table)

## 3. Automation

- [x] 3.1 Add workflow in `velikodniy/homebrew-tap` to detect latest cgt-tool release, fetch checksums, and update formula version/sha
- [x] 3.2 Allow manual dispatch, schedule, and repository_dispatch event; push commit with updated formula
- [x] 3.3 Add smoke install step in the workflow (brew install cgt-tool; `cgt-tool --version`)

## 4. Validation

- [x] 4.1 Run `openspec validate publish-homebrew-formula --strict`
- [x] 4.2 Capture release/version info used for the formula in the change notes (v0.1.0)
