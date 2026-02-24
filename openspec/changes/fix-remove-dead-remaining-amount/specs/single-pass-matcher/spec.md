## MODIFIED Requirements

### Requirement: Acquisition lot tracking

The system SHALL track acquisition lots with original amount, price, expenses, cost offset, consumed, reserved, and in_pool fields. The `remaining_amount` field is removed; `available()` and `held_for_adjustment()` SHALL use `original_amount` directly.

#### Scenario: Available shares calculation uses original amount

- **WHEN** an acquisition lot is created with amount 100
- **THEN** `available()` returns `original_amount - consumed - reserved - in_pool`

#### Scenario: Held for adjustment uses original amount

- **WHEN** an acquisition lot has consumed 30 of 100 shares
- **THEN** `held_for_adjustment()` returns `original_amount - consumed` (i.e., 70)
