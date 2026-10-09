## Why

A review of Fill from MusicBrainz found ways it can write a wrong release's tags without asking, and ways a good release is missed or a lookup fails for one bad request. The title search never looks at the artist, so another artist's "Greatest Hits" with the same track count can score close to 100 %; a single candidate is filled at once however weak its match; and on a release whose first medium is a DVD, files are matched to the DVD's tracks and given the wrong disc number.

## What Changes

- The album artist becomes a match signal: confidence weighs how many of the album artist's words the release's artist credit shares, and a release found by title must share at least one when both are known. The artist is compared locally; nothing more is sent to MusicBrainz.
- A single candidate is filled directly only when its confidence is at least 80 %, the editor's own threshold for a strong match; a weaker one is shown in the release list, so the user decides.
- Disc numbers count audio media only, both when matching a file by its disc and track number and when filling Disc number, consistent with Total discs.
- With Include cover on, the cover is not fetched when the chosen release matches fewer than half the files, since it would be left out; the matched tracks are filled at once.
- A release that cannot be read is left out of the candidates instead of failing the whole lookup; the lookup fails only when none can be read.
- The ISRC search reads every page of results instead of the first 100, and the title search compares the album's track count with the release's audio tracks, not its release-wide count that includes video and data media.
- A cover or lookup result that arrives after its lookup was cancelled or replaced is dropped, and every path that closes or replaces the editor stops a running lookup.
- A stale doc line on `Release::total_tracks` is removed.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `tag-editing`: Finding the album's release (artist on the title path), Match confidence (artist signal), Choosing among releases (weak single candidate listed), Matching tracks to the release (audio disc numbering), Lookup failures (an unreadable release is skipped).

## Impact

- `crates/qobuz-core/src/musicbrainz.rs`: `DiskAlbum` gains `artist`; `confidence`, `fitting_titles`, `match_track`/`track_fields`, `search_isrcs` (paging), `lookup` (skip unreadable releases); tests and fixtures.
- `crates/qobuz-gui/src/app/tag_editor.rs`: `disk_album` passes the artist; `found` lists a weak single candidate; `take` skips the cover when the album fields are kept; the strong-match threshold moves here from the view.
- `crates/qobuz-gui/src/app.rs`: lookup messages carry a lookup id instead of the album id; one helper stops the lookup wherever the editor closes or is replaced.
- No new dependency; no config change.
