# Tasks

## 1. Core: finding and scoring releases

- [x] 1.1 Add `artist` to `DiskAlbum`, an `artist_likeness` (containment) and an artist signal (weight 15) in `confidence`; verify with tests: another artist's same-title release scores below 80 %, a fuller credit naming the album artist scores as a match, and the existing "release without ISRCs" test reaches 100 % with the artist
- [x] 1.2 In `fitting_titles`, count the audio tracks listed on a search result's media and drop releases whose credit shares no word with the album artist when both are known; verify with tests for a CD + DVD-Video release found by title and an ABBA "Greatest Hits" left out
- [x] 1.3 Page `search_isrcs` by `offset` until `count`, at most 5 pages; verify with a unit test on the page plan and the existing ISRC fixture test
- [x] 1.4 In `lookup`, skip a release that cannot be read and fail only when none can be; verify with a unit test on the collecting helper
- [x] 1.5 Number audio media only in `match_track` and `track_fields`; verify with a test on a DVD-Video + CD release matched by position and by ISRC
- [x] 1.6 Remove the stale doc line on `Release::total_tracks`; verify `cargo doc`-visible text reads once

## 2. GUI: choosing, cover and lookup lifetime

- [x] 2.1 Pass the album artist in `TagEditor::disk_album`, move `STRONG_MATCH` to `app/tag_editor.rs`, and list a single candidate below it instead of filling; verify with app tests for a 90 % candidate filled and a 35 % one listed
- [x] 2.2 In `take`, fill at once without fetching the cover when fewer than half the files match; verify with an app test that no cover fetch starts and the matched track is filled
- [x] 2.3 Replace album ids on lookup messages with a lookup id, add `App::stop_lookup` and call it wherever the editor closes or is replaced and before a new lookup; verify with app tests that a late cover or lookup result from a cancelled lookup leaves a new lookup running
- [x] 2.4 Update the `musicbrainz.rs` and tag editor entries in `CLAUDE.md`; verify they match the code

## 3. Checks

- [x] 3.1 Run `cargo fmt`, `cargo clippy --workspace --all-targets` and `cargo test --workspace`, all clean
- [x] 3.2 Run `openspec validate harden-musicbrainz-fill --strict`
