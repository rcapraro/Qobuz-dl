## 1. Core: surface hi-res flag

- [x] 1.1 In `crates/qobuz-core/src/models.rs`, add `#[serde(default)] pub hires: bool` and `#[serde(default)] pub hires_streamable: bool` to `Album` (after `label`).
- [x] 1.2 Add the same two fields to `Track` (after `album`).
- [x] 1.3 Add `pub fn is_hires(&self) -> bool { self.hires_streamable || self.hires }` to both the `Album` and `Track` impl blocks.
- [x] 1.4 Add/extend a `models` unit test asserting `hires`/`hires_streamable` deserialize and that `is_hires()` returns true when either flag is set and false when both are absent.

## 2. GUI: display models

- [x] 2.1 In `crates/qobuz-gui/src/app.rs`, change `AlbumResult` to `{ id, title, artist, cover, hires }` (replace `label`, add `hires: bool`).
- [x] 2.2 Add `TrackResult { id, title, artist, hires }` and change `SearchPayload.tracks` to `Vec<TrackResult>`. Leave `artists` as `(String, String)`.

## 3. GUI: populate results

- [x] 3.1 In `crates/qobuz-gui/src/app/tasks.rs::do_search`, populate `AlbumResult`/`TrackResult` from `title`, `artist_name()`, and `is_hires()` instead of building a joined label. Keep the existing cover-selection logic.

## 4. GUI: render rows

- [x] 4.1 In `crates/qobuz-gui/src/app/view/search.rs`, refactor `add_row` to take a title, optional artist subtitle, and `hires: bool`; render the fill element as a two-line `column![ bold(title), text(artist).size(TEXT_SM) ]`.
- [x] 4.2 Add a trailing "Hi-Res" `Badge` (when `hires`) before the Add button, built like `app/view/queue.rs:120` with `style::badge(bg, fg)` and an accent pair from `style::accents(theme)`. Add `use iced_aw::widget::badge::Badge;`.
- [x] 4.3 Update `album_result_row` and `result_row` call sites (tracks pass title/artist/hires; artists pass name as title, no subtitle, `hires=false`).

## 5. Verify

- [x] 5.1 `cargo fmt` and `cargo clippy --workspace` clean.
- [x] 5.2 `cargo test -p qobuz-core` passes (including the new models test).
- [x] 5.3 `cargo run -p qobuz-gui`: search a hi-res album and a CD-only one; confirm the Hi-Res badge shows only on hi-res rows and that title (bold) and artist (secondary) render on separate lines for albums and tracks.
