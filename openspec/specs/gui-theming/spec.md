# gui-theming

## Purpose

Provide consistent, switchable, and persisted visual theming across all GUI screens, along with uniform control sizing and aligned form layout, so the application presents a coherent and polished user interface.

## Requirements

### Requirement: Application-wide theme

The application SHALL render all screens using a single active theme derived from a centralized palette, so that colors, surfaces, and text styling are consistent across the Settings, Search, and Queue screens.

#### Scenario: Consistent theming across screens

- **WHEN** the user navigates between the Settings, Search, and Queue screens
- **THEN** the background, surface, accent, text, and border colors are drawn from the same active theme on every screen

#### Scenario: Themed controls

- **WHEN** any screen renders buttons, text inputs, and containers
- **THEN** those controls use the active theme's styling rather than raw framework defaults

### Requirement: Light and dark theme switch

The application SHALL provide a user-facing control to switch between a light theme and a dark theme, and SHALL apply the selected theme to the entire application immediately without requiring a restart.

#### Scenario: Toggle to dark theme

- **WHEN** the light theme is active and the user activates the theme switch
- **THEN** the entire application re-renders using the dark theme immediately

#### Scenario: Toggle to light theme

- **WHEN** the dark theme is active and the user activates the theme switch
- **THEN** the entire application re-renders using the light theme immediately

### Requirement: Persisted theme preference

The application SHALL persist the user's selected theme and SHALL restore that theme on the next launch. When no preference has been stored, the application SHALL start with a defined default theme.

#### Scenario: Preference restored on restart

- **WHEN** the user selects a theme and later relaunches the application
- **THEN** the application starts with the previously selected theme

#### Scenario: Default on first run

- **WHEN** the application launches with no stored theme preference
- **THEN** the application starts with the defined default theme

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

### Requirement: Aligned form layout

The application SHALL align form fields, their labels, and associated action buttons on a consistent layout grid, using shared spacing and padding constants, so that controls within a screen line up along consistent edges and baselines.

#### Scenario: Fields and labels align

- **WHEN** the Settings screen renders its labeled input rows
- **THEN** the labels and inputs align to consistent column edges and the rows use uniform vertical spacing

#### Scenario: Uniform spacing constants

- **WHEN** any screen lays out rows and columns of controls
- **THEN** the spacing and padding between elements are drawn from shared layout constants rather than ad-hoc per-widget values

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
