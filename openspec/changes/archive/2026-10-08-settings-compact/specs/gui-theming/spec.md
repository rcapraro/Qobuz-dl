# Spec Delta

## MODIFIED Requirements

### Requirement: Semantic accent colors

Each Catppuccin accent color SHALL carry one meaning across the application:
mauve for brand, blue for the primary action and active selection, green for
success, yellow for in-progress, red for error, teal for audio quality,
lavender for a page's identity panel, such as an opened album's header, and
sapphire for help and explanatory panels.
Section and card headers SHALL use neutral surfaces rather than an accent.

#### Scenario: Card headers are neutral

- **WHEN** any screen renders a titled card
- **THEN** the card header uses a neutral surface color, not an accent color

#### Scenario: Success and error are consistent

- **WHEN** a success or an error is shown, whether as a status message or as a queue item badge
- **THEN** success uses the green accent and error uses the red accent everywhere

#### Scenario: Quality uses its own accent

- **WHEN** a hi-res or delivered-quality indicator is shown
- **THEN** it uses the teal accent, which no status or action uses

#### Scenario: Meaning holds in both themes

- **WHEN** the user switches between the light and dark themes
- **THEN** each accent keeps the same meaning, drawn from the active flavor's palette

#### Scenario: Identity panels use their own accent

- **WHEN** a screen shows the identity panel of what it displays, such as an opened album's header
- **THEN** the panel uses a soft lavender wash with a lavender border, and lavender is used for no status, action, or quality indicator

#### Scenario: Help panels use their own accent

- **WHEN** a help panel is shown
- **THEN** it uses a soft sapphire wash with a sapphire border, and sapphire is used for nothing else, including button hover states
