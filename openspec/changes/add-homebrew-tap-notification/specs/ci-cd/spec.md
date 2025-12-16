## ADDED Requirements

### Requirement: Homebrew Tap Release Notification

The system SHALL notify the Homebrew tap repository when a new release is published, triggering an automated formula update.

#### Scenario: Release triggers tap notification

- **WHEN** a GitHub release is published in `velikodniy/cgt-tool`
- **THEN** a `repository_dispatch` event with type `cgt-tool-release` is sent to `velikodniy/homebrew-tap`
- **AND** the event payload includes the release tag name

#### Scenario: Tap workflow receives notification

- **WHEN** the tap repository receives a `cgt-tool-release` dispatch event
- **THEN** the existing `update-cgt-tool.yml` workflow runs with the provided tag
- **AND** the Homebrew formula is updated to the new release version

#### Scenario: Missing token prevents notification

- **WHEN** the `TAP_REPO_TOKEN` secret is not configured
- **THEN** the notification step fails with a clear error message
- **AND** the release itself is not affected (notification is a separate job)
