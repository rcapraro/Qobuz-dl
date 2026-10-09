## Context

The fill pipeline is `lookup` (barcode → ISRCs → title, then up to 5 releases read in full and scored by `confidence`) in `qobuz-core`, then `TagEditor::found`/`take`/`cover_fetched` in the GUI, with the lookup task's abort handle on `App.musicbrainz`. Lookup messages carry the album id, which tells apart editors but not two lookups on the same editor.

## Goals / Non-Goals

**Goals:** stop a wrong release from being written without the user's say, keep a good release from being missed for counting reasons, and keep one bad request or a late message from ending or corrupting a lookup.

**Non-Goals:** no new request kinds or fields sent to MusicBrainz (the artist is compared locally, so "Considerate use of MusicBrainz" still holds); no change to how the picker looks.

## Decisions

**Artist likeness is containment, not Jaccard.** MusicBrainz credits often name more people than Qobuz's album artist (composer, orchestra, conductor). `artist_likeness` is the larger of the two shares of shared words, so "Richard Hickox" against "Vaughan Williams; London Symphony Orchestra, Richard Hickox" is 1.0 while "Queen" against "ABBA" is 0. Weight 15: with no ISRCs, a same-title, same-count, official release by another artist scores 35/50 = 70 %, below the 80 % strong-match line, and with a matching label 40/55 = 73 %. Left out when either side has no artist, like the other signals. *Alternative:* `title_likeness` (Jaccard) — rejected, it scores a correct but fuller credit at 0.25.

**Title path filters on the artist too, only when both are known.** `fitting_titles` drops a release whose credit shares no word with the album artist. Search results carry `artist-credit`, so no extra request. A release search result lists every medium, so its audio track count is summed from `media` (`Release::audio_tracks_listed`), falling back to `track-count` when media are absent. *Alternative:* add `AND artist:(…)` to the query — rejected, it would send the artist to MusicBrainz and miss credits worded differently.

**ISRC ranking keeps the release-wide count as a tie-break.** A recording search lists only the matched medium of each release, so its audio count cannot be known there; the count only orders releases holding the same number of ISRCs, and each one read is scored on its audio tracks. Pages are read by `offset` until `count` is reached, at most 5 pages (500 recordings) to bound a lookup at the 1 request/s pace.

**Audio ordinal as disc number.** `Release::audio_media_numbered()` yields `(n, medium)` with `n` counting audio media from 1. `match_track` searches only audio media and compares the file's disc with `n`; `track_fields` writes `n` as Disc number. A track found by ISRC on a non-audio medium is therefore unmatched, which is right: it is not on the album's discs.

**Strong-match threshold shared.** `STRONG_MATCH` (80) moves from the view to `app/tag_editor.rs`, and the view imports it, so the badge colour and the direct-fill decision agree.

**Skip the cover when the album is kept.** `take` computes the fill first; when fewer than half match, it fills at once and never enters `Lookup::Cover`. Otherwise a failed cover fetch still fills nothing, as the spec requires.

**Lookup id instead of album id on messages.** `App.musicbrainz` becomes `Option<RunningLookup { id, handle }>` with ids from a counter. `Message::MusicBrainz` and `MusicBrainzCover` carry the id, and a message whose id is not the running lookup's is dropped before anything is changed. `App::stop_lookup()` aborts and clears it; it is called from Cancel, Close, opening an editor, a failed read, the editor closing after a save, and before a new lookup starts. *Alternative:* only reorder the cover handler's checks — rejected, it would not tell apart two lookups on the same album.

**Unreadable release skipped.** In `lookup`, a failed release read is logged and skipped; when every read failed, the first error is returned, so an offline lookup still reports the failure.

## Risks / Trade-offs

- [A correct release whose credit shares no word with Qobuz's album artist, such as a transliterated name] → It is not dropped on the barcode or ISRC paths, and is only scored lower; on the title path it is not offered. Accepted: the title path is the weakest evidence, and writing another artist's tags is the worse failure.
- [Up to 5 ISRC pages add up to 4 s at 1 request/s] → Only for albums with more than 100 matching recordings.
