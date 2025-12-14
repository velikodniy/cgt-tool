## ADDED Requirements

### Requirement: Proceeds Breakdown Parity

The PDF disposal calculation SHALL display net proceeds using quantity × unit price minus sale expenses, matching the calculated proceeds and staying aligned with the plain text formatter.

#### Scenario: Sell with sale expenses

- **WHEN** a disposal includes sale expenses
- **THEN** the proceeds line is shown as `<quantity> × £<unit price> - £<sale expenses> = £<net proceeds>`
- **AND** `<net proceeds>` equals `(unit price × quantity) - sale expenses` rounded using the currency’s minor units (per the currency policy).

#### Scenario: Sell without sale expenses

- **WHEN** sale expenses are zero
- **THEN** the proceeds line is shown as `<quantity> × £<unit price> = £<net proceeds>`
- **AND** `<net proceeds>` equals `unit price × quantity` rounded using the currency’s minor units (per the currency policy).
- **AND** the proceeds line omits the `- £0` term when sale expenses are zero.
