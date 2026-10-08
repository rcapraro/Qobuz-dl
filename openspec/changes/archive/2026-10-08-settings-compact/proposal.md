# Proposal

## Why

At the 949px window height this display allows, the Settings screen no longer fits: Options and Save settings sit below the fold. Three things account for the extra height:
- The App ID and App secret fields are stacked.
- The credentials card carries a helper sentence under its buttons.
- Options take two lines.

The concurrency range of 1–16 is also wider than useful for one account's downloads. It is enforced only by the Settings widget, so a saved config can hold any value and the engine will use it.

This follow-up to `settings-forms` partly reverses that change's layout, by decision on 2026-10-08.

## What Changes

- **Credentials row:** App ID and App secret share one row. App ID is a narrow input on the label column, followed by "App secret:" and a full-width secret input.
- **Credentials help:** the helper sentence under Auto-detect / Check signing moves into the credentials help panel, which gains a line explaining Check signing (it already explains Auto-detect).
- **Options row:** Options fit on one line:
  - Quality on the label column;
  - Concurrency next to it, with the stepper narrowed to fit one or two digits;
  - then the two checkboxes.

  "Notify when downloads finish" becomes "Notify when done". The options help panel keeps the full explanation.
- **Concurrency range:** it becomes 1–10, owned by core.
  - A shared maximum bounds the Settings control.
  - A saved value above 10 is clamped to 10 when the config is loaded, so the engine never runs more than 10 tracks at once.
  - The options help says 1–10.
- **Help panels:** all four Settings help panels (credentials, account, options, templates) open in one shared panel: a soft sapphire wash with a sapphire border. Before, three were plain text in the card body, the same colour as the form, and the template help was a neutral card. Sapphire becomes the "info" accent role. Sky was considered first but is already the primary button's hover colour.

No new settings. Saving, persistence and the unsaved-changes hint are unchanged.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `downloader-gui`:
  - "Settings screen": concurrency range 1–10, plus a scenario for a saved value above 10.
  - "Labeled settings fields": the first input of each row starts on the label column, and the secret follows an inline label.
  - "One primary action per settings card": button explanations live in the card's help panel.
  - "Options card layout" is removed and replaced by an added "Single-line Options card" requirement, since the spec tooling cannot rename its "Options on two lines" scenario.
- `desktop-notifications`: "Notification setting" names the option "Notify when done".
- `gui-theming`: "Semantic accent colors" adds sapphire for help panels.
- `template-help`: "Template syntax help" shows its content in the shared help panel instead of a neutral card.
- `settings-help`: adds "Help panels stand apart from the form".

## Impact

- `crates/qobuz-core/src/config.rs`: a `MAX_CONCURRENCY` constant and a clamp in `Config::load`, with a test.
- `crates/qobuz-gui/src/app/view/settings.rs`: the credentials row, the Options row, a narrower stepper using the shared maximum, and the shorter checkbox label.
- `crates/qobuz-gui/src/app/help.rs`: credentials help gains Check signing; options help says 1–10 and names "Notify when done"; every panel is wrapped in a shared `help_panel` container.
- `crates/qobuz-gui/src/style.rs`: a `sapphire` palette colour, an `Accents::info()` role and a `help_panel` container style.
- `crates/qobuz-gui/src/app/view/mod.rs`: the now-unused `card` helper is removed.
- The README Settings screenshot goes out of date.
