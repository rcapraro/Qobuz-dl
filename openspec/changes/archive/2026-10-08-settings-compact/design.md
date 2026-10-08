# Design

## Context

See proposal.md for motivation and the two delta specs for requirements. The Settings view (`app/view/settings.rs`) builds each row with `style::labeled_row`, a 130px label column (`LABEL_WIDTH`) followed by the control. After `settings-forms`:
- the credentials card is two stacked `labeled_row`s, then an indented button row, then an indented helper sentence;
- Options is a `labeled_row("Quality:", …)` holding Concurrency, then an indented row of checkboxes.

The concurrency bound exists only as the `1..=16` range of the iced_aw `NumberInput`. `Config` has no bound, and the engine applies only `.max(1)`.

## Goals / Non-Goals

**Goals:**
- Save settings visible without scrolling at the default window size, if the saved rows allow it (checked on screen, not promised).
- One source of truth for the concurrency range.

**Non-Goals:**
- Replacing the stepper with a dropdown. The narrower stepper was chosen.
- Wrapping Options onto two lines at narrow window widths. iced rows don't wrap; one line is designed for the default size.
- Changing the Account or File organization cards.

## Decisions

### Concurrency range in core
`qobuz-core/src/config.rs` gains `pub const MAX_CONCURRENCY: usize = 10`. `Config::load` clamps a parsed `concurrency` to `1..=MAX_CONCURRENCY` before returning it. The lower bound matches the engine's existing `.max(1)`. The Settings `NumberInput` range becomes `1..=MAX_CONCURRENCY`, and the help text says 1–10.

The clamp happens in `load`, before the GUI takes its saved-config snapshot, so a clamped value doesn't show the unsaved-changes hint. The file on disk keeps the old value until the next save of any kind. That is harmless because every load clamps it again.

*Alternative:* clamping in the engine as well. Rejected because `load` is the only way a config enters the app, and the widget bounds every edit after that. A second clamp would be a second source for the same rule.

### Credentials on one row
The first `labeled_row("App ID:", …)` holds a row:
- the `app_id` input at a fixed `APP_ID_WIDTH`, sized for its 9-digit value plus padding, roughly 140px;
- `text("App secret:")`;
- the `app_secret` input at `Fill`.

The button row stays indented under the label column via `labeled_row("", …)`. The helper sentence is removed from the card. `help::credentials_help` gains a "Check signing" term next to the existing `app_id`/`app_secret` terms, saying that it sends one signed request with the current credentials, needs a sign-in, and reports whether signatures are accepted.

### Options on one line
`labeled_row("Quality:", row![pick_list, "Concurrency:", NumberInput, checkbox, checkbox])`. The old `horizontal_space` separator is dropped, so the controls sit left-aligned with uniform spacing. The `NumberInput` narrows from 120px to `CONCURRENCY_WIDTH`, roughly 72px: wide enough for two digits plus its spinner, confirmed on screen.

Width budget at the default window, about 960px of card content:

| Element | Approx. width |
|---|---|
| label column | 130 + 8 spacing |
| Quality pick list | ~222 |
| "Concurrency:" | ~103 |
| stepper | ~72 |
| "Embed cover art" checkbox | ~148 |
| "Notify when done" checkbox | ~165 |
| spacing | ~5 gaps of `SPACE_MD` |
| **Total** | **≈ 905** |

If the on-screen check shows less room than estimated, `SPACE_MD` gaps between the checkboxes become `SPACE_SM` before anything else changes.

### Label wording
The checkbox reads "Notify when done". The options help entry is renamed to match and keeps its full explanation (posts a desktop notification when a batch ends while the app is in the background, not after Cancel).

### Help panels: one sapphire callout
Added after implementation review: the credentials, account and options help were plain columns pushed into the card body, the same `surface0` colour as the form, while the template help was a neutral card. All four now go through `help::help_panel`, a container styled by `style::help_panel`:
- a 12% sapphire wash with a 60% sapphire border, radius 8, `SPACE_MD` padding;
- modelled on `style::hero`, the lavender identity panel;
- the template help's card header becomes a "Template help" heading inside the panel.

Sapphire gets its own role, `Accents::info()`, rather than reusing an existing accent:
- Sky was the first choice but is already the primary button's hover colour (`style.rs`), so it would carry two meanings.
- Lavender is reserved for identity panels.
- Blue means primary and active.

`view::card` lost its only caller and is removed. `card_el` stays for the Settings cards.

## Risks / Trade-offs

- [This reverses part of settings-forms' layout on the day it shipped] → The labeled-fields and one-primary-per-card requirements are kept and reworded, not dropped. Only the stacking and the visible helper sentence go.
- [A user with concurrency 11–16 silently gets 10] → Intentional (decision 2026-10-08). The Settings control shows 10, so the change is visible where the setting lives.
- [The Options line overflows at narrow widths] → Accepted. The estimate leaves about 55px of margin at the default size.
- [Explanations behind "?" are less discoverable] → The help toggle is on the same card header, and both buttons keep self-describing labels.
