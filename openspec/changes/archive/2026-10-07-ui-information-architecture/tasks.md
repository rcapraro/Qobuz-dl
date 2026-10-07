# Tasks

## 1. Design system: semantic accents and button variants

- [x] 1.1 Add role methods to `Accents` in `style.rs` (`brand`, `primary`, `success`, `progress`, `error`, `quality`) and verify with a unit test that each role maps to the expected hue in both Latte and Macchiato
- [x] 1.2 Make `card` use a neutral `surface1` header with `text`, drop the `head` parameter from `card`/`card_el`, update every call site in `view/settings.rs`, `view/search.rs`, and `help.rs`, and verify `cargo build -p qobuz-gui` succeeds
- [x] 1.3 Let `styled_button`/`action_button`/`secondary_button` take `impl IntoFragment<'a>` labels, add `compact_button`, and verify `cargo build -p qobuz-gui` succeeds
- [x] 1.4 Replace the hand-built buttons in `view/queue.rs` (Retry failed, Clear queue, Cancel, per-row Retry and Remove) with the shared builders, and verify `grep -n "button(" crates/qobuz-gui/src/app/view` finds no direct `button(` calls outside `style.rs`
- [x] 1.5 Map queue badges to roles (tagging uses progress) and split `done · <quality>` into a green done badge plus a teal quality badge, and verify by running the app and downloading a track

## 2. Typed status messages

- [x] 2.1 Introduce `StatusKind`/`Status` with constructors, make `App.status` an `Option<Status>`, convert every `self.status = …` site to a kind, and verify `cargo build -p qobuz-gui` succeeds
- [x] 2.2 Render the status bar with a per-kind icon and accent outline plus a dismiss control for errors (`Message::DismissStatus`). Check each glyph renders in bundled Inter and fall back to ASCII where it does not. Verify with an update test that `DismissStatus` clears an error
- [x] 2.3 Verify visually in both themes that info, progress, success, and error are distinguishable (for example, trigger a search, a failed sign-in, and Settings saved)

## 3. Navigation

- [x] 3.1 Reorder tabs to Search, Queue, Settings and start on `Screen::Search`, and verify by launching the app with `cargo run -p qobuz-gui`
- [x] 3.2 Add a pure `remaining(queue)` and the `Queue (N)` label, and verify unit tests cover an empty queue, only done or failed tracks (no count), and a mix of queued, downloading, and tagging tracks
- [x] 3.3 Replace the theme toggle glyph labels with plain text ("Light theme" / "Dark theme") and verify both states render

## 4. Search screen: omnibox and setup prompt

- [x] 4.1 Add a pure `classify(input)` returning `Add`, `BadUrl`, or `Search { bare_id }`, and verify unit tests for a Qobuz album URL, a playlist URL, a non-Qobuz URL, a word ("Radiohead"), a number ("1989"), and a multi-word query
- [x] 4.2 Collapse the two bars into one field ("Search albums and tracks, or paste a Qobuz URL"), remove `url_input`/`UrlChanged`/`AddUrl`, route submit through `classify`, and verify that pasting a URL enqueues while typing text searches
- [x] 4.3 Render the "Add as ID" row above results when `bare_id` is set, and verify that it enqueues only when activated
- [x] 4.4 Add a pure `setup_gap(config, signed_in)` and the setup prompt with an "Open Settings" button, remove the now-redundant startup "go to Settings" status strings, and verify unit tests for missing credentials, signed out, and complete

## 5. Documentation and checks

- [x] 5.1 Update the README usage flow and the screen list in `CLAUDE.md` (tab order, single search field) and verify the described steps match the running app
- [x] 5.2 Run `cargo fmt --check`, `cargo clippy --workspace --all-targets -- -D warnings`, and `cargo test --workspace`, and verify all pass

## Workflow follow-up

- Archive this change and sync `downloader-gui` and `gui-theming` before proposing `search-pagination`, whose delta modifies the same Search requirement.
