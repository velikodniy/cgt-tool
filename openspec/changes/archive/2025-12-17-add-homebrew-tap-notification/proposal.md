# Change: Add Homebrew Tap Release Notification

## Why

Currently, updating the Homebrew formula in `velikodniy/homebrew-tap` requires manual workflow triggers after each cgt-tool release. This creates delay between releases and formula availability, and risks forgetting to update the tap.

## What Changes

- Add a new GitHub Actions workflow (`notify-tap.yml`) in `cgt-tool` that triggers on published releases
- The workflow dispatches a `repository_dispatch` event to `velikodniy/homebrew-tap` with the release tag
- Requires a Personal Access Token (`TAP_REPO_TOKEN`) stored as a repository secret

## Impact

- Affected specs: `ci-cd` (adds cross-repo notification requirement)
- Affected code: `.github/workflows/notify-tap.yml` (new file)
- External dependency: Requires `TAP_REPO_TOKEN` secret with `repo` scope for cross-repo dispatch
