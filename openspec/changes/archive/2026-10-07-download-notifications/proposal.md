# Proposal

## Why

A large batch takes minutes. Users switch to other work meanwhile and have
to keep checking back to learn whether it finished or failed. A desktop
notification when the batch ends removes that polling.

## What Changes

- When a download batch finishes on its own, the app posts an OS desktop
  notification: "Downloads finished" with the number of tracks downloaded,
  or "Downloads finished with errors" with the downloaded and failed counts.
- No notification is posted for a batch the user cancelled, since they just
  acted in the app, or while the app window has focus, since the status line
  already reports the outcome there.
- A "Notify when downloads finish" setting in the Options card turns this
  off. It is on by default and persisted with the other settings.
- On macOS, notifications come from the installed Qobuz-dl app when it is
  installed. A development build has no installed bundle, so its
  notifications are attributed to Finder.
- A failure to post a notification never affects the batch, the queue, or
  the status line. It is only logged.
- **New dependency:** `notify-rust` 4.18 in `qobuz-gui`. iced 0.13 has no
  notification API, and notifications go through a different system service
  on each OS (`NSUserNotificationCenter` on macOS, WinRT toasts on Windows,
  D-Bus on Linux).

## Capabilities

### New Capabilities

- `desktop-notifications`: notifying the user outside the app window when
  background work ends, and the setting that controls it.

### Modified Capabilities

None. The new setting is persisted through the existing "Persist settings"
requirement of `app-configuration`, which already covers every non-secret
setting.

## Impact

- `qobuz-gui`: `Cargo.toml` and `Cargo.lock` (the `notify-rust` dependency),
  `app.rs` (a window-focus subscription, notification on
  `DownloadsFinished`), `app/tasks.rs` (posting off the UI thread), `main.rs`
  or `app::run` (one-time macOS application identity),
  `app/view/settings.rs` (the toggle).
- `qobuz-core`: `config.rs` gains `notify_on_finish: bool`, which defaults to
  true. Existing config files load unchanged because `Config` is
  `#[serde(default)]`.
- Linux builds gain `zbus` 5 next to the `zbus` 4.4 already in the
  dependency tree.
- Final change of the UI and feature roadmap. Follows `queue-grouping`.
