# Tasks

## 1. Spike: confirm MusicBrainz responses

- [x] 1.1 Fetch and save trimmed JSON fixtures under `crates/qobuz-core/src/musicbrainz/testdata/`: a barcode search with 2+ hits, a full release lookup (design's `inc` list) of a short album, a full lookup of a release with more than 25 tracks, and a recording search by ISRC (`isrc:A OR isrc:B …`; the ISRC lookup endpoint returns no releases), all sent with the design's User-Agent and one second apart. Verify by checking that each file parses as JSON and that no secret or user data is in it
- [x] 1.2 From the fixtures, confirm or correct in design.md: the long release lists all its tracks, recordings carry their ISRCs, recording → work → composer relations are present, release-group `first-release-date`, `secondary-types` and `genres` are present, and whether `front-1200` exists for an old Cover Art Archive upload. Verify by having no risk in design.md still marked "checked in the spike"
- [x] 1.3 Confirm that Qobuz `album/get` returns `upc` for a known album and record its format (leading zeros, length) in design.md. Verify by noting the album id and value checked

## 2. Core: release data and mapping

- [x] 2.1 Add `upc: Option<String>` to `models::Album`. Verify with a deserialization test that reads it and one that tolerates its absence
- [x] 2.2 Add the `musicbrainz` module with its serde models for search, release and ISRC responses (`#[serde(default)]` throughout). Verify with tests parsing each 1.1 fixture
- [x] 2.3 Implement track matching (ISRC, then disc and position) and the mapping of a release onto one `Option<TagFields>` per edited track, as in design.md. Verify with fixture tests for every scenario of "Fields filled from a release" and "Matching tracks to the release": original date, composer from works, genre spelling and release-group fallback, compilation off, ISRC/explicit/copyright/comment absent, ISRC match despite renumbering, unmatched track
- [x] 2.4 Implement the confidence score with the design's weights, including the redistribution when no track has an ISRC. Verify with tests: the exact edition outranks a partial one with bonus tracks, scores lie in 0–100, and a release with no ISRC signal still ranks by the other signals
- [x] 2.5 Add the process-wide pacing gate and the HTTP layer (User-Agent, `with_retry`). Verify with a `tokio::time::pause` test where three gated calls start at least one second apart
- [x] 2.6 Implement the lookup (barcode search, ISRC fallback with release tally capped at 5, full lookup of each candidate) emitting progress and returning candidates, plus the Cover Art Archive fetch (`front-1200`, 404 → none, with the fallback 1.2 decided). Verify with a unit test of the pure ISRC tally function and a test that a 404 cover maps to "no cover"; the network path is checked in 5.2
- [x] 2.7 Re-export the module's public API in `lib.rs` and document the module in CLAUDE.md's qobuz-core list (barcode → ISRC fallback, pacing gate, confidence, mapping). Verify that `cargo doc -p qobuz-core --no-deps` builds without warnings

## 3. GUI: editor state and lookup

- [x] 3.1 Extract `TagEditor::fill(Vec<Option<TagFields>>)` from `reset_to_qobuz` and make Reset call it. Verify that the existing reset tests pass unchanged, and add a test where unmatched tracks (`None`) keep their track fields while matched tracks agree on an album field
- [x] 3.2 Add `include_cover`, `lookup: Option<Lookup>` and the messages for starting a fill, progress, candidates, choosing, cancelling and the fetched cover; gate Save, Reset and Fill on no lookup. Verify with update tests: Save unavailable while fetching, a single candidate fills and sets a status with the confidence, several candidates enter `Choosing`, cancelling leaves every field as before, a failure sets an error status and changes no field, and a cover result goes through `replace_cover` with the Cover art setting
- [x] 3.3 Add the `tasks.rs` wrapper: the lookup stream bridged with `iced::stream::channel`, started with `Task::abortable`, its handle kept with the editor; drop events for an album that isn't open. Verify with an update test that closing the editor during `Fetching` drops the handle and a late result changes nothing

## 4. GUI: view

- [x] 4.1 Add Fill from MusicBrainz and the Include cover checkbox to the editor header beside Reset to Qobuz, using `style.rs` builders, with the lookup progress on the status line and the header width kept still as labels change (as the Close/Discard slot does). Verify by running the app and watching the header while a lookup runs
- [x] 4.2 Draw the release picker in place of the track list when `Choosing`: one row per candidate (title, date, country, label, format, track count, confidence) sorted by confidence, a choose action per row and Cancel, colored by semantic role and using only glyphs Inter has. Verify by running the app on an album whose barcode matches several releases
- [x] 4.3 Document Fill from MusicBrainz, the picker, its confidence and Include cover in the README's tag editor section, and the editor's lookup state in CLAUDE.md's qobuz-gui paragraph. Verify that the README describes what 4.1 and 4.2 show

## 5. Integration

- [x] 5.1 Run `cargo fmt --check`, `cargo clippy --workspace --all-targets` and `cargo test --workspace`. Verify that all three pass
- [x] 5.2 In the running app, download an album, fill from MusicBrainz with and without Include cover, save, and reopen the editor. Verify that the saved values read back, ReplayGain and explicit are unchanged, and the cover follows the option; also test with the network off and confirm the failure is reported and no field changes. Done on "Richard Hickox conducts Vaughan Williams" (36 files, found by title): without cover, the saved files gained MusicBrainz album, album artist, artist, date, label and 33 titles, kept copyright, ISRC, composer, genre and numbering, and kept the Qobuz cover; with cover, all 36 got the Cover Art Archive front resized to the 600 px setting and no tag changed. The files carry no ReplayGain or explicit tags, so unlisted-tag survival rests on the existing tag_edit tests; the offline case is covered by `a_failed_lookup_changes_nothing` and `a_failed_cover_changes_nothing` rather than cutting the network. The files were restored from a backup afterwards

## 6. Follow-ups from testing and review

Found while testing in the app (the Hickox compilation) and in code review, after groups 1–5 were planned.

- [x] 6.1 Offer an ISRC-path release only when it holds at least half of the edited tracks' ISRCs, and match by disc and track number only on a release with the album's track count. Verified by `isrc_ranking_drops_releases_holding_under_half` and `positions_are_not_matched_on_a_release_laid_out_otherwise`
- [x] 6.2 Leave the ISRC, label and status confidence signals out when MusicBrainz has no value for them. Verified by `a_release_without_isrcs_is_ranked_on_the_rest` and `a_release_without_label_or_status_is_ranked_on_the_rest`
- [x] 6.3 Add the title fallback (`release:(words) AND mediums:N`, kept when the release has the album's track count and at least half its title's words). Verified by `title_search_keeps_releases_laid_out_like_the_album` and by the live test finding the Hickox release by title
- [x] 6.4 Report "no release matches this album" as an error in the status line. Verified by `nothing_found_is_an_error`
- [x] 6.5 Let the user cancel a lookup at any step (Cancel lookup, the picker's Cancel), and block editor input while one runs. Verified by `a_search_can_be_cancelled`, `a_cover_download_can_be_cancelled` and `input_waits_for_a_lookup`
- [x] 6.6 Fill only the matched tracks, never the album fields or cover, when fewer than half the files match. Verified by `few_matches_leave_the_album_fields_and_cover`
- [x] 6.7 Count only audio media for Total discs and the release's track count, and take a search result's release-wide count. Verified by `a_video_disc_is_not_a_disc_of_the_album`, `a_search_result_counts_the_whole_release` and `isrc_ranking_ties_on_the_whole_release_track_count`
- [x] 6.8 Upgrade iced to 0.14 and iced_aw to 0.14.1, whose text honours `Wrapping::None` (iced #2723), and keep search titles on one line with the full title on hover. Verified by `cargo clippy` and `cargo test` passing on 0.14; the screens are checked in 4.1 and 4.2
