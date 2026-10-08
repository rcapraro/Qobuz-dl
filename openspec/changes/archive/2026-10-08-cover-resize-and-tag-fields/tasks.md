# Tasks

## 1. Verify before building

- [x] 1.1 Confirm from the `image` 0.24.9 source in the cargo registry: the decoder `Limits` API, that `imageops::resize` accepts an `Rgb32F` buffer, and how to read dimensions without a full decode. Record any difference from design.md in the design.

## 2. Core

- [x] 2.1 Add `CoverSize` (`Px400`, `Px500`, `Px600`; default `Px600`) and `Config::cover_size` with `#[serde(default)]`. Add tests: an old config with `embed_art: false` loads as Off, and one without `cover_size` loads as 600 px. Verify with `cargo test -p qobuz-core config`.
- [x] 2.2 Add `image` (`default-features = false`, `jpeg` and `png`) and `jpeg-encoder` 0.7 to `qobuz-core`, and check that `Cargo.lock` still has a single `image` 0.24.9.
- [x] 2.3 Create `artwork.rs` with `prepare_cover(bytes, size)`, following design.md. Tests:
  - 600×600 at 600 px → byte-identical; 450×450 at 500 px → byte-identical (not enlarged)
  - 600×600 at 400 px → 400×400; 600×450 at 500 px → 500×375
  - 1-pixel checkerboard at half size → about 188 in sRGB
  - the output JPEG is 4:4:4, checked by reading its SOF component sampling factors
  - garbage input → input returned unchanged

  Verify with `cargo test -p qobuz-core artwork`.
- [x] 2.4 In `engine.rs`, make `fetch_cover` run `prepare_cover` on `spawn_blocking`, and cache the result. Keep the race against cancellation and the existing cancel test passing. Verify with `cargo test -p qobuz-core engine`.
- [x] 2.5 Add `Album::copyright` and `Job::disc_track_total`, the latter computed in `resolve`. Test single-disc, multi-disc, and no track list.
- [x] 2.6 In `tagging.rs`, write the full release date, label, copyright, total discs, total tracks and explicit flag, as design.md describes. Add read-back tests for FLAC and MP3, plus MP4 if a minimal fixture is practical. Each test checks the explicit flag through the concrete tag (Vorbis `ITUNESADVISORY`, ID3 `TXXX:ITUNESADVISORY`, MP4 `rtng`). Verify with `cargo test -p qobuz-core tagging`.

## 3. GUI

- [x] 3.1 Replace the Embed cover art checkbox with a Cover art pick list (Off, 400 px, 500 px, 600 px; labels kept short so the Options card stays on one line) mapped onto `embed_art` and `cover_size`. Keep the Options card on one line at the default window size, and make sure changing it shows the unsaved-changes hint.
- [x] 3.2 Update the Options help text for the Cover art choices.

## 4. Docs

- [x] 4.1 Update `CLAUDE.md`: add `artwork.rs` to the module list, and the cover step (source choice, capped resize) to the per-track data flow.

## 5. Checks

- [x] 5.1 Run `cargo fmt --check`, `cargo clippy --workspace --all-targets` and `cargo test --workspace`, and verify all pass with no new warnings.
- [x] 5.2 Download one album at 400 px and one at 600 px, and check the embedded art's dimensions and appearance in a player or `metaflac --export-picture-to`.
