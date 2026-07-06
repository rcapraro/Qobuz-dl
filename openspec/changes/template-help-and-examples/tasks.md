## 1. State & messages (app.rs)

- [x] 1.1 Add `show_template_help: bool` field to `struct App`; initialize `false` in `App::new`
- [x] 1.2 Add `Message::ToggleTemplateHelp` and `Message::CopyTemplate(String)` variants (near `ToggleTheme`)
- [x] 1.3 Add update arm `ToggleTemplateHelp` → flip `self.show_template_help`, `Task::none()`
- [x] 1.4 Add update arm `CopyTemplate(s)` → `iced::clipboard::write(s)` (returns the clipboard `Task`)

## 2. Style helper (style.rs)

- [x] 2.1 Add a `mono(content) -> Text` helper: `iced::Font::MONOSPACE`, size `TEXT_SM` (for template/token strings)

## 3. Example data & row builder (app.rs)

- [x] 3.1 Add `const FOLDER_EXAMPLES: &[(&str, &str)]` and `const TRACK_EXAMPLES: &[(&str, &str)]` (label, template) using only real placeholders (see design.md)
- [x] 3.2 Add `example_row(template: &str, apply: Message) -> Element` showing monospace template (fill width) + `secondary_button("Copy", Message::CopyTemplate(...))` + `secondary_button("Apply", apply)`, spacing/`align_y` per `style` constants

## 4. Help content (app.rs)

- [x] 4.1 Add a `template_help() -> Element` (or inline block) rendering: placeholder list with descriptions (`albumartist, artist, album, title, year, container, bit_depth, sampling_rate, explicit, composer, tracknumber`), the `{key:0N}` zero-padding note, and rules (folder `/` = subfolders, illegal chars sanitized, unknown → empty). Use `section()` headings + `mono()` for tokens
- [x] 4.2 Add "Folder examples" and "Track examples" subsections built from the const slices via `example_row`, passing `Message::FolderFormatChanged`/`TrackFormatChanged` as the apply message respectively

## 5. Wire into settings_view (app.rs)

- [x] 5.1 In the "File organization" section, add a `secondary_button` toggling help with label `"Show template help"`/`"Hide template help"` based on `self.show_template_help`
- [x] 5.2 When `self.show_template_help`, append the help + examples content below the existing folder/track inputs and preview

## 6. Verification

- [x] 6.1 `cargo build --workspace` and `cargo clippy --workspace` clean
- [x] 6.2 `cargo run -p qobuz-gui`: on Settings, toggle help on/off; confirm placeholders/rules render and examples show in monospace
- [x] 6.3 Click **Apply** on a folder example and a track example → the corresponding input and the live preview update
- [x] 6.4 Click **Copy** on an example → paste elsewhere confirms the template string is on the clipboard
