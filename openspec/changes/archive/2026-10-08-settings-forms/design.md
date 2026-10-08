# Design

## Context

`App.config` is edited in place by every Settings message and is what downloads use, so edits take effect immediately and are persisted only by a save. There are three kinds of save path:

- the explicit `SaveSettings` handler, which calls `config.save()` directly;
- `CredentialsDetected`, which also calls `config.save()` directly;
- `save_config()`, used by the theme toggle, `LoggedIn`, and adopting a working fallback secret (after `SigningChecked` and at the end of a download batch in `DownloadsFinished`).

Every save writes the whole `Config`, so an implicit save also persists any unrelated edits pending at that moment. This change does not alter that. It only makes the state visible. See proposal.md for motivation and specs/downloader-gui/spec.md for the requirements.

## Goals / Non-Goals

**Goals:**
- An unsaved-changes signal that is always truthful, whichever path saved.
- One aligned form column across all four cards.

**Non-Goals:**
- Changing which actions save implicitly, or making them save only their own field.
- Warning on quit with unsaved changes. This needs window close interception and is out of the polish scope.
- Field validation (e.g. checking that the directory is writable).

## Decisions

### Snapshot compare, not a dirty flag
`App` gains `saved_config: Option<Config>`, set from the loaded config in `from_parts` and replaced with `Some(self.config.clone())` after every successful save. "Dirty" is `saved_config != Some(config)`, which needs `#[derive(PartialEq)]` on `Config`. `Quality` already derives it and the other fields are std types.

*Alternative:* a `dirty: bool` set by each edit message. It is rejected because it can't detect an edit that is later reverted, and every new setting would have to remember to set it. The snapshot is derived state, which matches the existing rule that `signed_in` and theme are derived, never stored twice.

When `Config::load` fails and the app falls back to defaults, the snapshot is `None`: the file on disk still holds the unreadable config, so the app counts as unsaved and Save stays available to replace it. Before this change Save was always enabled. Recording the defaults as saved would have disabled it, leaving the broken file in place and the load error on every launch.

### One save helper
`SaveSettings` and `CredentialsDetected` stop calling `config.save()` directly and go through a helper that saves and, on success, refreshes the snapshot. Its `Result` lets each caller keep its own status message ("Settings saved.", "Credentials detected and saved…"). `save_config()` becomes a thin wrapper over it. A single success path guarantees the "Implicit save clears the hint" scenario: no caller can save without refreshing the snapshot.

### Save row
`Save settings` stays the primary `action_button`. It is disabled with `on_press_maybe(dirty.then_some(...))` and followed by the text "Unsaved changes" in the `progress()` (yellow) accent role. Yellow already means "pending / not settled" in the app, and red would suggest an error. When the settings are clean the hint text is absent, and no reserved space is needed since it is the last item in the row.

### Labels via `labeled_row`
The credentials become two stacked `labeled_row`s, `App ID:` and `App secret:`, instead of two side-by-side inputs. Side by side, a shared label column is impossible. The token input becomes `labeled_row("Token:", …)` with Sign in / Sign out after the input in the same row.

The credential button row is indented to the input column by placing it in a `labeled_row("", …)`, so the buttons line up under the fields they act on. Placeholders stay as format hints (`paste your user_auth_token`), not as labels.

### Button variants
- Check signing becomes `secondary_button` and keeps `Length::Shrink` for its longer label.
- Sign in stays `styled_button`, which is the primary style.
- Sign out keeps the secondary style.

These are existing builders, so the gui-theming rule that every button comes from a shared variant still holds. The helper text becomes one sentence covering both actions, e.g. "Auto-detect fetches app_id and app_secret from the Qobuz web player; Check signing verifies them (requires sign-in)."

### Options layout
The card body becomes a column of two rows:
1. `labeled_row("Quality:", pick_list)`, then a horizontal space, `Concurrency:` and the `NumberInput`.
2. The two checkboxes, indented to the input column the same way as the credential buttons.

## Risks / Trade-offs

- [The `PartialEq` derive on a core type exists only for the GUI] → `PartialEq` is a harmless, conventional derive with no serde or behavior impact.
- [An implicit save persists unrelated pending edits] → This is existing behavior. The hint now disappears truthfully when it happens, instead of silently reporting nothing. Changing it is a non-goal.
- [Stacking the credentials adds one row of height] → The Settings tab already scrolls, and labeled fields are worth the row.
