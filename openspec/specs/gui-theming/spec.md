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
