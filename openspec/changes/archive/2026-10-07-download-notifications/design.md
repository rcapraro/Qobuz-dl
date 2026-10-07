# Design

## Context

- `Message::DownloadsFinished` is the single place a batch ends. It already
  computes `was_cancelled`, the error count, and the done count for the
  status line.
- `App` has no subscription today. iced 0.13.1 provides
  `iced::event::listen_with(fn(Event, event::Status, window::Id) ->
  Option<Message>)`, `iced_core::window::Event::{Focused, Unfocused}`, and
  `Application::subscription`.
- `Config` is `#[serde(default)]` at struct level, so a new field loads with
  its default from existing config files.
- The `notify-rust` 4.18.1 and `mac-notification-sys` 0.6.15 sources were read
  from crates.io:
  - Default features are `["z"]` (zbus with `async-io`) on Linux/BSD. macOS
    uses `mac-notification-sys`. Windows uses `tauri-winrt-notification`.
  - On macOS, `Notification::show()` calls `sendNotification`, which can wait
    on delivery. Its blocking behaviour is not documented, so it is treated as
    blocking on every platform.
  - On macOS the delivering application is process-global and set once.
    `set_application` runs inside a `Once` and returns `AlreadySet` after
    that. If nothing sets it, the first send uses
    `get_bundle_identifier_or_default("use_default")`, which is
    `com.apple.Finder`.
  - notify-rust re-exports `get_bundle_identifier_or_default(app_name)` and
    `set_application(bundle_id)` on macOS.
- The packager identifier is `com.qobuzdl.qobuz-dl` and the product name is
  `Qobuz-dl`.

## Goals / Non-Goals

**Goals:**
- Build the notification text and the decision to notify in pure functions
  with unit tests. The OS call is a thin, fire-and-forget task.
- Never block the UI thread on the OS notification service.

**Non-Goals:**
- Clicking the notification to focus the app or open the download folder.
  `wait_for_click` would hold a thread per notification.
- Per-track notifications, sounds, or notifications for anything other than
  batch completion.
- Badging the dock icon or the taskbar.

## Decisions

### Decide and word the notification in pure code

```rust
struct BatchOutcome { downloaded: usize, failed: usize, cancelled: bool }
fn notification(outcome: &BatchOutcome, enabled: bool, focused: bool)
    -> Option<(String /*summary*/, String /*body*/)>
```

It returns `None` when the batch was cancelled, notifications are disabled,
or the window is focused. The counts cover only the batch that ended:
`spawn_downloads` records the batch's track ids in `App.batch`. The
whole-queue counts used by the status line would include tracks finished by
earlier batches, so a 12-track batch could be reported as "17 tracks
downloaded". The summary is "Downloads finished" when `failed ==
0` and "Downloads finished with errors" otherwise. The body is
"12 tracks downloaded" or "10 downloaded, 2 failed", with singular forms.
`DownloadsFinished` builds the outcome from the counts it already computes.

### Post off the UI thread, fire-and-forget

`tasks::notify(summary, body)` runs `tokio::task::spawn_blocking(move ||
Notification::new().appname("Qobuz-dl").summary(..).body(..).show())`. Any
error, including a failed join, is logged with `tracing::warn!` and mapped
to `()`. `Task::perform(tasks::notify(..), |_| Message::Notified)`, where
`Notified` is a no-op. The app already enables iced's `tokio` feature, so
`spawn_blocking` is available.

*Alternative:* `show_async`. Rejected because it only exists on XDG and on
macOS behind the `preview-macos-un` feature, so it is not portable.

### macOS application identity, set once at startup

In `app::run`, before the iced application starts, and only on macOS:

```rust
let bundle = notify_rust::get_bundle_identifier_or_default("Qobuz-dl");
let _ = notify_rust::set_application(&bundle);
```

When the packaged app is installed, this resolves to the real bundle
identifier and notifications carry the app's name and icon. A development
binary has no registered bundle, so it resolves to `com.apple.Finder`, the
same identity as leaving it unset, but chosen explicitly and only once. It
runs once, before any send, which is what `set_application`'s `Once`
requires.

*Alternative:* hard-code `com.qobuzdl.qobuz-dl`. Rejected for development
builds, where that identifier is not registered. `set_application` then
returns `CouldNotSet` and its `Once` is already used, so it cannot be retried
with a fallback. What macOS does with later sends in that state is not
documented. Resolving through `get_bundle_identifier_or_default` avoids that
state.

### Focus tracking

`App.window_focused: bool` starts `true`, because the window opens focused.
A `subscription()` maps `Window(Focused)` and `Window(Unfocused)` to
`Message::WindowFocus(bool)` through `event::listen_with`. Nothing else is
subscribed, so the subscription stays this one listener.

### Setting

`Config.notify_on_finish: bool` defaults to `true` through `Default` (the
struct is `#[serde(default)]`). The Options card gets a checkbox, "Notify
when downloads finish", beside "Embed cover art", sent through
`Message::NotifyToggled(bool)`. It is saved by the existing "Save settings",
like the other options. The options help panel gains one line explaining it.

## Risks / Trade-offs

- [Linux builds gain `zbus` 5 next to the `zbus` 4.4 already in the tree]
  → Larger binary and longer build on Linux only. The `d` feature would
  instead require the system `libdbus` at build and run time, which is worse
  for the AppImage and deb.
- [Windows toasts from an unpackaged binary use a fallback identity, such as
  PowerShell's] → The toast still shows. The NSIS install registers the app,
  which is the case that matters.
- [macOS may never show a notification if the user denied permission] →
  Posting is fire-and-forget and logged. The status line still reports the
  outcome.
- [The window can lose focus during a batch and regain it before the end] →
  Focus is read when the batch ends, which matches whether the user is
  looking at the app at that moment.

## Migration Plan

No migration: `notify_on_finish` is absent from existing config files and
defaults to on. Rollback is a revert, including `Cargo.lock`, per the
project's rule that a dependency change ships with its lockfile.
