# Spec Delta

## ADDED Requirements

### Requirement: Help panels stand apart from the form
Every Settings help panel (API credentials, Account, Options and File organization templates) SHALL be shown in one shared panel style, a tinted and bordered container distinct from the card body it opens in, so explanations never read as part of the form.

#### Scenario: Help panel is set apart
- **WHEN** the user opens any card's help
- **THEN** the help content appears inside a tinted, bordered panel within that card, visibly distinct from the card's fields and buttons

#### Scenario: All help panels look the same
- **WHEN** the user opens the help of two different cards
- **THEN** both panels use the same tint, border and padding
