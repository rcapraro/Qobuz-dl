# Spec Delta

## MODIFIED Requirements

### Requirement: Consistent control sizing

The application SHALL size interactive controls of the same role consistently. All buttons of the same variant SHALL share the same height, internal padding, and minimum width, and all single-line text inputs SHALL share the same height and padding. Every button SHALL use one of the shared variants: primary, secondary, or compact. The compact variant is for actions inside a list row and for small header or inline controls, such as the theme toggle and the status dismiss control.

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

- **WHEN** a list row or a queue group header offers an action, such as adding a search result or removing an album group
- **THEN** that action is rendered with the compact variant, and all compact actions share the same height and padding

#### Scenario: Theme toggle uses the compact variant

- **WHEN** the header shows the theme toggle
- **THEN** it is rendered with the compact variant, not as a full-width secondary button
