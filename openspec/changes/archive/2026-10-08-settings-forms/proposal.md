# Proposal

## Why

Settings is the one screen with an explicit Save, but it never says whether there is anything to save. Edits apply in memory at once and are lost on quit unless Save is pressed. The form itself is inconsistent:
- The credential and token inputs have placeholders only, so a filled field loses its label.
- The credentials card has two primary buttons while Sign in, the one action every new user needs, is styled like the rest.
- The Options card packs four controls into one row.

This is the first change of the 2026-10-08 polish pass (settings-forms → chrome-and-layout → queue-row-redesign → search-affordances).

## What Changes

- Show an "Unsaved changes" hint next to Save settings while the settings differ from what is persisted. Save is enabled only in that state. Saving, including the saves that already happen implicitly (theme toggle, sign-in, auto-detect, fallback-secret adoption after a signing check or a download batch), clears the hint.
- Give `app_id`, `app_secret` and the token input visible labels in the same aligned label column as File organization, so a field keeps its label once it is filled.
- Allow at most one primary button per settings card:
  - Auto-detect stays primary and Check signing becomes secondary.
  - Sign in is primary and Sign out is secondary.
  - The credentials helper text describes both buttons.
- Lay out the Options card on two lines: Quality and Concurrency, then the Embed cover art and Notify checkboxes.

No new settings, no autosave, no persistence changes.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `downloader-gui`: adds requirements for the unsaved-changes indicator, labeled settings fields, one primary action per settings card, and the Options card layout. The existing "Settings screen" requirement is untouched, and its "Configure and save" scenario still holds.

## Impact

- `crates/qobuz-gui/src/app.rs`: a saved-config snapshot on `App`, refreshed on load and on every successful save. The two direct `config.save()` calls go through one helper.
- `crates/qobuz-gui/src/app/view/settings.rs`: labeled rows, button variants, Options layout, the Save row hint.
- `crates/qobuz-core/src/config.rs`: `Config` derives `PartialEq` (all field types already support it). No serialization change.
- README settings screenshot becomes outdated.
