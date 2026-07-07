## Context

`qobuz-gui` is an `iced 0.13.1` Elm-architecture app (`App` / `Message` / `update` /
`view`). All UI is built from base `iced` widgets in `crates/qobuz-gui/src/app.rs`
with a custom design system (spacing, typography, hand-tuned light/dark `Palette`)
in `crates/qobuz-gui/src/style.rs`. Today:

- Navigation is a `row` of buttons (`nav_button`) driven by a `Screen` enum and
  `Message::Navigate`; the theme toggle and sign-in indicator share that row.
- `concurrency` is stored as a `String` on `App` and parsed + clamped to `1..=16`
  on every keystroke (`Message::ConcurrencyChanged(String)`).
- Template help is a `column` gated by a `show_template_help: bool` flag.
- Queue item status renders as plain `text` derived from `ItemStatus`.

`iced_aw` is the official companion crate offering higher-level widgets. It is
tightly version-coupled to `iced`, so version selection is the pivotal decision.

## Goals / Non-Goals

**Goals:**
- Replace navigation with a real tab bar; concurrency with a bounded number input;
  template help with a card; queue status with badges; group results/settings in
  cards.
- Preserve all existing behavior (auth, download engine, config persistence, copy/
  apply of template examples, live preview) and honor the existing custom theme in
  both light and dark modes.
- Confine changes to `qobuz-gui`.

**Non-Goals:**
- No changes to `qobuz-core` (API, config schema, download engine).
- No new download/search features; this is a presentation refresh only.
- No adoption of iced_aw widgets beyond `tabs`, `number_input`, `card`, `badge`.

## Decisions

### Decision: Pin `iced_aw = 0.12.2`
`iced_aw 0.12.2` declares `iced ^0.13.1` (with the `advanced` feature) — an exact
match for the project's `iced 0.13.1`. Newer iced_aw (0.13.x, 0.14.x) targets
`iced 0.14` and will not compile here.
- **Alternatives considered:** iced_aw 0.11.0 also targets iced 0.13.1 but is
  older; latest (0.14.1) targets iced 0.14 (rejected — would force an iced upgrade).
- **Consequence:** add `advanced` to the existing `iced` features, and depend on
  `iced_aw = { version = "0.12", default-features = false, features = ["tabs", "number_input", "card", "badge"] }`. Feature-gating keeps compile time and binary size down (the release build is size-optimized).

### Decision: Tabs replace the button row; header row keeps global controls
`iced_aw::Tabs` keyed by the existing `Screen` enum as the tab id; `Message::Navigate`
becomes the on-select handler. The theme toggle and sign-in indicator, which live in
the current nav row, move to a slim header row rendered **above** the tab bar so
they remain always visible.
- **Alternatives considered:** placing global controls inside a tab (rejected —
  they are cross-cutting, not section-specific).

### Decision: `number_input` for concurrency, dropping the `String` field
`iced_aw::number_input` binds directly to `self.config.concurrency` (`usize`) with a
`1..=16` range and its own increment/decrement controls. The `App.concurrency: String`
field and the parse-and-clamp branch in `update` are deleted; `Message::ConcurrencyChanged`
carries a `usize`.
- **Consequence:** invalid/out-of-range input becomes structurally impossible rather
  than corrected after the fact.

### Decision: `Card` for the template help; toggle retained
The `template_help()` content is wrapped in an `iced_aw::Card`. The `show_template_help`
toggle stays (the `template-help` spec requires toggleable visibility); the live path
preview stays above the inputs. Copy/Apply example controls are unchanged.

### Decision: `Badge` for queue status; `Card` grouping for results/settings
`ItemStatus` maps to a colored `iced_aw::Badge` (e.g. success green for Done, danger
red for Error) beside each queue row; progress bars remain. Search result sections
(Albums/Tracks/Artists) and Settings sub-sections are wrapped in `iced_aw::Card`s.
- **Alternatives considered:** `selection_list` for results — **rejected**: it is a
  single-select list with no per-row action buttons, incompatible with the existing
  per-row "Add" control. There is no `grid` feature in iced_aw 0.12, so cards are the
  grouping primitive.

### Decision: iced_aw widgets styled from the existing palette
iced_aw 0.12 uses iced 0.13's `Catalog`/style-closure model. Add small style helpers
in `style.rs` (e.g. tab / card / badge style fns) that read the current custom
`Palette` so the new widgets track light/dark like the rest of the UI.

## Risks / Trade-offs

- **iced_aw styling API differs from base iced** → Mitigation: write thin per-widget
  style closures in `style.rs`; verify visually in both themes during apply.
- **Extra dependency increases build time / binary size** → Mitigation: pin
  `default-features = false` and enable only the four needed features.
- **Tabs relayout changes the top-of-window structure** → Mitigation: keep the
  header row for global controls; verify no regression in sign-in/theme affordances.
- **`number_input` range differs from prior silent clamp behavior** → Mitigation:
  set range `1..=16` to match the previous clamp; existing configs already fall in
  range, and `#[serde(default)]` covers older files.

## Migration Plan

Presentation-only, single crate. No data migration. Rollback = revert the
`qobuz-gui` diff and the `Cargo.toml` dependency lines; persisted config is
unaffected either way.

## Open Questions

None outstanding — scope, template treatment, and widget set were confirmed with the
user (broad refresh; template card + live preview).
