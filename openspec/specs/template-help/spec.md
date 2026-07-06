# template-help Specification

## Purpose
TBD - created by archiving change template-help-and-examples. Update Purpose after archive.
## Requirements
### Requirement: Template syntax help

The Settings screen SHALL provide in-app help for the path-template syntax that documents the supported placeholders, the zero-padding modifier, and the key rendering rules, using only placeholders that the download engine actually populates.

#### Scenario: Placeholders are documented

- **WHEN** the user opens the template help
- **THEN** the help lists the supported placeholder tokens (`albumartist`, `artist`, `album`, `title`, `year`, `container`, `bit_depth`, `sampling_rate`, `explicit`, `composer`, `tracknumber`) each with a short description

#### Scenario: Syntax rules are documented

- **WHEN** the user reads the template help
- **THEN** it explains the `{key}` token form, the `{key:0N}` zero-padding modifier for numeric values, that a `/` in the folder template creates nested subfolders, that illegal filename characters are sanitized, and that unknown placeholders render as empty text

### Requirement: Toggleable help visibility

The template help SHALL be hidden by default and SHALL be shown or hidden by a user-activated control, so the Settings screen stays uncluttered.

#### Scenario: Help hidden by default

- **WHEN** the Settings screen is first shown
- **THEN** the template help content is not displayed and a control to show it is available

#### Scenario: Toggle help

- **WHEN** the user activates the help toggle
- **THEN** the help content becomes visible, and activating the toggle again hides it

### Requirement: Example templates

The Settings screen SHALL present a curated set of example templates, including at least two folder examples and two track examples, each displayed with its literal template string.

#### Scenario: Examples are listed

- **WHEN** the user views the template help/examples area
- **THEN** it shows named example templates for both the folder format and the track format, each showing the exact template string

### Requirement: Copy an example template

Each example template SHALL provide a control that copies its literal template string to the operating-system clipboard.

#### Scenario: Copy to clipboard

- **WHEN** the user activates the copy control for an example
- **THEN** that example's template string is written to the OS clipboard

### Requirement: Apply an example template

Each example template SHALL provide a control that sets it as the current folder or track format, updating the corresponding input and the live preview immediately.

#### Scenario: Apply a folder example

- **WHEN** the user activates the apply control for a folder example
- **THEN** the folder format input is set to that example's string and the path preview updates to reflect it

#### Scenario: Apply a track example

- **WHEN** the user activates the apply control for a track example
- **THEN** the track format input is set to that example's string and the path preview updates to reflect it

