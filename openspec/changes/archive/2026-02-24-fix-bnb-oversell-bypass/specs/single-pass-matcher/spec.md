## ADDED Requirements

### Requirement: Disposal requires sufficient holding

Before the matching cascade (Same Day → B&B → S104), the matcher MUST verify that the seller holds enough shares to cover the disposal. The holding is the sum of same-day acquisitions not yet matched and the Section 104 pool quantity for that ticker.

If the sell quantity exceeds the holding, the matcher MUST return an error explaining that the disposal exceeds the current holding. B&B matching MUST NOT satisfy a disposal that lacks a backing holding.

Per HMRC CG51590 Example 1, B&B determines cost basis for a valid disposal — it does not enable disposing of shares the taxpayer does not hold.

#### Scenario: B&B does not rescue a sell with zero holding

- **WHEN** a disposal occurs for a ticker with 0 shares held
- **AND** an acquisition of the same ticker exists within 30 days after the disposal
- **THEN** processing MUST fail with an error indicating the disposal exceeds the current holding

#### Scenario: B&B with valid holding succeeds

- **WHEN** a disposal occurs for a ticker with sufficient shares in the S104 pool
- **AND** an acquisition of the same ticker exists within 30 days after the disposal
- **THEN** the disposal MUST be matched via B&B using the later acquisition's cost
- **AND** the S104 pool MUST NOT be reduced (per HMRC CG51590 Example 1)

#### Scenario: Partial holding insufficient for full disposal

- **WHEN** a disposal of N shares occurs for a ticker with fewer than N shares held
- **THEN** processing MUST fail with an error indicating the disposal quantity exceeds the holding
