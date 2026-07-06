## Context

The path-template feature is fully functional but undocumented in-app. Grounding from the code:

- **Engine** (`crates/qobuz-core/src/template.rs`): `{key}` tokens resolved against `TemplateContext.values` (any key, not a fixed whitelist); `{key:0N}` zero-pads numeric values (`resolve_token` L114-126, `numeric` L128-135). `render_path` splits folder templates on `/` into sanitized segments; `render_segment` sanitizes the track name as one segment. Sanitization strips `/ \ : * ? " < > |` + control chars, collapses whitespace, trims `.`, caps at 200 chars, never empty.
- **Real placeholders** set in `engine.rs::build_context` (L248-279): `albumartist, artist, album, title, year, container, bit_depth, sampling_rate, explicit` (`" [E]"`/`""`), `composer` (only when present), `tracknumber`. There is **no** `discnumber` token (multi-disc handled by a `Disc N` subfolder outside templating). The preview (`app.rs::template_preview`) sets the same keys except `composer`.
- **Defaults** (`config.rs`): folder `"{albumartist} - {album} ({year}) [{container}] [{bit_depth}B-{sampling_rate}kHz]"`, track `"{tracknumber:02}. {artist} - {title}"`.
- **Settings UI** (`app.rs` `settings_view`, "File organization" block ~L561-577): `section("File organization")`, `dir_row`, two `labeled_row`s wrapping `field_input(...).on_input(Message::FolderFormatChanged/TrackFormatChanged)`, then a live `preview` container. Format-changed messages set `self.config.*` immediately so the preview re-renders live.
- **Reusable pieces**: `style.rs` helpers (`section` is actually a local fn in `app.rs`, plus `action_button`, `secondary_button`, `field_input`, `labeled_row`, spacing/typography constants). `Message::Add(Reference)` shows buttons carry payloads. `ToggleTheme` (app.rs L177-182) shows the bool-field + toggle-button pattern. `update` already returns `Task<Message>` and non-none tasks.
- **Clipboard**: none today; `iced` 0.13 provides `iced::clipboard::write(String) -> Task<Message>`.

## Goals / Non-Goals

**Goals:**
- Document the real, working template syntax in-app (placeholders + `{key:0N}` + rules).
- Keep it out of the way by default (toggle), consistent with existing UI patterns.
- Offer curated example templates with per-example **Copy** (to clipboard) and **Apply** (set the field) actions.

**Non-Goals:**
- No changes to the template engine or the placeholder set.
- No documenting of a `discnumber` token (it doesn't exist).
- No rich-text/markdown rendering, no external links, no persistence of the help-open state.

## Decisions

### 1. Toggleable, always-in-Settings help block
Add `show_template_help: bool` to `struct App` (default `false`), a `Message::ToggleTemplateHelp` arm that flips it, and a `secondary_button("Show template help" / "Hide template help", Message::ToggleTemplateHelp)` in the "File organization" section. When true, render the help + examples column below it.

*Why:* mirrors the existing `ToggleTheme` state pattern exactly; no new widget type needed (iced 0.13 has no expander in use). *Alternative* — always-visible help — rejected as it clutters the (already scrollable) Settings screen.

### 2. Help content as static monospace text
Render the placeholder list and syntax rules as `text(...)`. Use a monospace font (`iced::Font::MONOSPACE`) for the template/token strings so `{tracknumber:02}` reads clearly. Add a small `style` helper `mono(text) -> Text` (font = MONOSPACE, size = `TEXT_SM`) and reuse the `section()` helper for headings.

*Why:* simplest thing that renders cleanly and stays theme-aware. The content is authored from the verified real placeholder set so nothing documented is dead.

### 3. Example templates as static data + a row builder
Define two `const` slices in `app.rs` (near the other view helpers): `FOLDER_EXAMPLES: &[(&str, &str)]` and `TRACK_EXAMPLES: &[(&str, &str)]` (label, template). Render each via a helper `example_row(template: &str, apply: Message) -> Element` showing: the template in monospace (fill width), a `secondary_button("Copy", Message::CopyTemplate(template.into()))`, and a `secondary_button("Apply", apply)`.

Curated examples (all use only real placeholders):
- Folder: `"{albumartist}/{album} ({year})"`, `"{albumartist} - {album} [{container}]"`, `"{albumartist}/{album} ({year}) [{bit_depth}B-{sampling_rate}kHz]"`.
- Track: `"{tracknumber:02} - {title}"`, `"{tracknumber:02}. {artist} - {title}"`, `"{artist} - {title}{explicit}"`.

*Why static consts:* the set is fixed and tiny; no config/persistence needed. *Payload-carrying messages* follow the `Message::Add(Reference)` precedent.

### 4. Copy vs Apply message wiring
- **Copy:** new arm `Message::CopyTemplate(String) => iced::clipboard::write(s)` — returns the clipboard `Task` directly (first clipboard use; `update` already returns `Task`).
- **Apply:** reuse existing `Message::FolderFormatChanged(String)` / `TrackFormatChanged(String)`. The `example_row` for folder passes `Message::FolderFormatChanged(t.into())` as its `apply`; the track rows pass `TrackFormatChanged`. No new apply message, and the live preview updates for free.

*Why:* minimal surface area; apply already has correct behavior (sets config + live preview).

## Risks / Trade-offs

- **Documented placeholder drifts from engine** → author help strictly from `engine.rs::build_context`; note that `composer` shows empty in the live preview (preview omits it) so users aren't surprised. Optionally add `composer` to the preview context for consistency (small, optional follow-up).
- **Clipboard unavailable on some Linux setups** → `iced::clipboard::write` is best-effort and returns a `Task`; failure is silent and non-fatal, acceptable for a convenience action. Apply provides a no-clipboard fallback.
- **Help lengthens the scrollable Settings column** → mitigated by the default-hidden toggle.
- **Monospace font availability** → `iced::Font::MONOSPACE` is a generic family iced maps to an available monospace; safe.

## Open Questions

- Whether to also add `composer` to `template_preview` so applied examples using `{composer}` preview non-empty. Proposed: leave preview as-is for this change (out of scope), but call it out. Decide during apply if trivial.
