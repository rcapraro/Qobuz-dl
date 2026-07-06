## Why

The folder/track path templates in Settings accept `{placeholder}` tokens and a `{key:0N}` zero-padding modifier, but nothing in the app documents which placeholders exist or how the syntax works — users must read the source or guess. A concise in-app reference plus ready-made example templates they can copy or apply makes the feature discoverable and lowers the barrier to customizing output paths.

## What Changes

- Add an in-app **template syntax help** section to the Settings "File organization" area: the list of supported placeholders with short descriptions, the `{key:0N}` zero-padding modifier, and the key behaviors (folder `/` creates subfolders; illegal characters are sanitized; unknown placeholders render empty).
- Show the help behind a compact **Show/Hide help** toggle so the Settings screen stays uncluttered by default (following the existing theme-toggle state pattern).
- Add a small curated set of **example templates** (a few folder examples and a few track examples), each rendered in a monospace style.
- Add a **Copy** action per example that writes the template string to the OS clipboard.
- Add an **Apply** action per example that sets it as the current folder or track format (reusing the existing format-changed messages, so the live preview updates immediately).

## Capabilities

### New Capabilities
- `template-help`: In-app documentation of the path-template syntax and a set of copyable/appliable example templates in the Settings screen.

### Modified Capabilities
<!-- None. openspec/specs/ currently contains only `gui-theming`; the file-organization/template behavior itself is unchanged (no engine changes). This adds a new, self-contained help/examples capability in the GUI. -->

## Impact

- **Code:** `crates/qobuz-gui/src/app.rs` — extend the "File organization" section of `settings_view` (~lines 561-577), add a `show_template_help: bool` field to `struct App`, add `Message` variants (`ToggleTemplateHelp`, `CopyTemplate(String)`), and add update arms (the copy arm returns `iced::clipboard::write(...)`; apply reuses `FolderFormatChanged`/`TrackFormatChanged`). Likely small additions to `crates/qobuz-gui/src/style.rs` (a monospace help/example text helper) and a place for the example-template data.
- **No engine/core changes:** the template engine (`crates/qobuz-core/src/template.rs`) and the real placeholder set (`engine.rs::build_context`) are unchanged; the help documents existing behavior. No new crates (clipboard is built into `iced` 0.13).
- **Presentation-only:** no change to authentication, search, download, tagging, or path rendering.
