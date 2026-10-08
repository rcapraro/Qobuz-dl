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

- **WHEN** a list row offers an action, such as retrying or removing a queued track
- **THEN** that action is rendered with the compact variant, and all compact actions share the same height and padding

#### Scenario: Theme toggle uses the compact variant

- **WHEN** the header shows the theme toggle
- **THEN** it is rendered with the compact variant, not as a full-width secondary button

## ADDED Requirements

### Requirement: Status line emphasis by kind
The status line SHALL outline itself in an accent colour only for an error message, using the error accent. Info, progress and success messages SHALL sit on a neutral surface with no accent outline, and their kind SHALL be shown by the colour of their icon.

#### Scenario: Error is outlined
- **WHEN** the status line shows an error message
- **THEN** it is outlined in the error accent

#### Scenario: Routine messages are not outlined
- **WHEN** the status line shows an info, progress or success message
- **THEN** it has no accent outline, and its icon carries the kind's accent colour

### Requirement: Tab bar emphasis
The tab bar SHALL mark the active tab with a primary-accent outline on a raised neutral surface, with its label in the regular text colour, and SHALL draw inactive tabs without a fill. No tab SHALL be drawn as a solid accent-coloured block. A hovered tab SHALL use the same raised surface, so the active tab keeps looking raised while the pointer is over it.

#### Scenario: Active tab
- **WHEN** a tab is active and not hovered
- **THEN** it is outlined in the primary accent on a raised neutral surface, and its label uses the regular text colour

#### Scenario: Active tab under the pointer
- **WHEN** the pointer is over the active tab
- **THEN** it keeps the raised neutral surface rather than reverting to an unfilled tab

#### Scenario: Inactive tabs
- **WHEN** a tab is not active and not hovered
- **THEN** it has no fill and its label uses the regular text colour

### Requirement: Shared content edge
Each tab's content SHALL be drawn directly beneath the tab bar without an enclosing bordered pane. On every tab, the content's left edge SHALL match the left edge of the header, status line and tab bar. Scrollable content SHALL reserve space for its scrollbar on the right only.

#### Scenario: No pane around tab content
- **WHEN** any tab is shown
- **THEN** its content is not enclosed in an additional bordered pane

#### Scenario: Fixed and scrolled content align
- **WHEN** a screen shows fixed controls above a scrollable list, such as the Search field above the results or the Queue header above its groups
- **THEN** the fixed controls and the scrolled content start at the same left edge, which is also the left edge of the header and tab bar
