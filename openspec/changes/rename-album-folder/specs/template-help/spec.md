# Spec Delta

## MODIFIED Requirements

### Requirement: Template syntax help

The Settings screen SHALL provide in-app help for the path-template syntax that documents the supported placeholders, the zero-padding modifier, and the key rendering rules, using only placeholders that the download engine actually populates. When shown, the help content SHALL be presented inside the same help panel as the other Settings help, a bordered container set apart from the settings fields, so it reads as a distinct, self-contained panel.

#### Scenario: Placeholders are documented

- **WHEN** the user opens the template help
- **THEN** the help lists the supported placeholder tokens (`albumartist`, `artist`, `album`, `title`, `year`, `container`, `bit_depth`, `sampling_rate`, `explicit`, `composer`, `tracknumber`) each with a short description

#### Scenario: Syntax rules are documented

- **WHEN** the user reads the template help
- **THEN** it explains the `{key}` token form, the `{key:0N}` zero-padding modifier for numeric values, that a `/` in the folder template creates nested subfolders, that illegal filename characters are sanitized, and that unknown placeholders render as empty text

#### Scenario: Help shown in a card

- **WHEN** the template help is visible
- **THEN** its content is rendered within the Settings help panel, a bordered container distinct from the surrounding settings fields, titled "Template help"

#### Scenario: Rename template explained

- **WHEN** the user reads the template help
- **THEN** it explains that the rename template only suggests a name for Rename folder in the queue, that its values come from the album's current file tags, and that it yields a single folder name, so a `/` does not create subfolders
