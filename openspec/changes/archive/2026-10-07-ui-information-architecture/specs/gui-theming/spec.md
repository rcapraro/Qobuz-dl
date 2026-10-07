# Spec Delta

## MODIFIED Requirements

### Requirement: Consistent control sizing

The application SHALL size interactive controls of the same role consistently. All buttons of the same variant SHALL share the same height, internal padding, and minimum width, and all single-line text inputs SHALL share the same height and padding. Every button SHALL use one of the shared variants: primary, secondary, or compact. The compact variant is for actions inside a list row.

#### Scenario: Buttons of the same variant match

- **WHEN** two buttons of the same variant are rendered on any screen
- **THEN** they have the same height, internal padding, and minimum width

#### Scenario: Text inputs match

- **WHEN** two single-line text inputs are rendered
- **THEN** they have the same height and internal padding

#### Scenario: Header actions share a variant

- **WHEN** the Queue screen header shows several actions at once, such as retry, clear, and start
- **THEN** each of those actions has the same height, internal padding, and minimum width as other buttons of its variant

#### Scenario: Row actions use the compact variant

- **WHEN** a list row offers an action, such as retrying or removing a queued track
- **THEN** that action is rendered with the compact variant, and all compact actions share the same height and padding

## ADDED Requirements

### Requirement: Semantic accent colors

Each Catppuccin accent color SHALL carry one meaning across the application:
mauve for brand, blue for the primary action and active selection, green for
success, yellow for in-progress, red for error, and teal for audio quality.
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
