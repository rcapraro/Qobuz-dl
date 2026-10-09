# Design

## Context

The tag editor already has one external source: Reset to Qobuz (`TagEditor::reset_to_qobuz`). It builds a `TagFields` for each track from the queue job (`EditorTrack::qobuz`), sets an album field when every track agrees, sets each track field Qobuz has a value for, and only changes the typed text, never the files. The editor is offered only for albums in this session's queue, so every `EditorTrack.job.album` holds the Qobuz album JSON that `engine::resolve` fetched. That JSON includes `upc`, but `models::Album` drops it. Downloaded files carry the ISRC (`tagging::fields_from`), and the editor reads it back as `Field::Isrc`.

A replacement cover arrives as bytes (`Message::CoverPicked`), is checked with `artwork::is_cover_image`, and is applied by `TagEditor::replace_cover`, which sets the resize option to the Cover art setting.

Every HTTP client in `qobuz-core` sends the placeholder User-Agent `qobuz-dl/0.1 (+https://github.com/)`. MusicBrainz requires a meaningful one and allows one request per second for each client; over that limit it answers 503.

## Goals / Non-Goals

**Goals:**
- Two requests in the usual case: a barcode search, then one full lookup of each candidate.
- The mapping from a release to fields, the track matching and the confidence are pure functions in `qobuz-core`, tested against saved MusicBrainz responses.
- Reuse Reset to Qobuz's merge, and the replacement-cover path, as they are.

**Non-Goals:**
- Fields outside the 17 the editor edits, such as MusicBrainz IDs, catalog number or release type.
- Acoustic fingerprinting (AcoustID): ISRCs and the barcode make it unnecessary.
- Free-text search of MusicBrainz by artist and title, and editing albums that aren't in the queue.
- Logging in to MusicBrainz or submitting data to it.

## Decisions

### A `musicbrainz` module in `qobuz-core`, beside `client.rs`
It owns its own `reqwest::Client` with User-Agent `qobuz-dl/<CARGO_PKG_VERSION> ( https://github.com/rcapraro/Qobuz-dl )` and its own serde models, which are `#[serde(default)]` throughout, like `models.rs`. It sits beside `QobuzClient` rather than inside it because it shares no auth, no signing and no base URL. The other clients' placeholder User-Agent is left alone, since Qobuz doesn't care.

### A pacing gate for the whole process
Requests to `musicbrainz.org` pass through one `static` gate (a `tokio::sync::Mutex<Option<Instant>>` behind a `OnceLock`). Each request waits until one second has passed since the previous one started. A gate per lookup would let two fills in a row break the limit. Requests are retried with `download::with_retry`, which already treats 503 as transient and backs off. The Cover Art Archive is a separate service with no such limit, so it does not go through the gate.

### Lookup flow
```
barcode (job.album.upc)
  └─ GET /ws/2/release?query=barcode:<upc>&fmt=json          → release ids (≤ 5)
     └─ none? ISRC fallback, one request for the whole album:
          GET /ws/2/recording?query=isrc:A OR isrc:B OR …&limit=100&fmt=json
          per release: count the distinct edited-track ISRCs among its recordings' ISRCs
          → only releases holding ≥ half of them; top 5 by that count, then by |release track-count − ours|
        └─ none? title fallback:
          GET /ws/2/release?query=release:(<title words>) AND mediums:<discs>&limit=25&fmt=json
          → only releases with the album's track count and ≥ half its title's words; most alike first
for each candidate:
  GET /ws/2/release/<mbid>?fmt=json&inc=recordings+isrcs+artist-credits+labels+
      release-groups+genres+recording-level-rels+work-rels+work-level-rels+artist-rels
→ candidates with confidence; 1 → fill, n → picker
```
Each candidate is fully looked up before the picker opens. ISRC coverage is the strongest confidence signal, and it needs the tracklist. Choosing a release then fills immediately, with no further request except the optional cover. Both paths take at most 6 requests, about 6 s at the pace limit, and the lookup reports which step it is on (below).

Spike findings (fixtures in `crates/qobuz-core/src/musicbrainz/testdata/`):
- `/ws/2/isrc/<isrc>?inc=releases` returns recordings **without** their releases, and adding `release-groups` is a 400. The recording search does return each recording's releases, and an `OR` of ISRCs keeps the fallback to one request. For a much-reissued album the response is about 1 MB.
- A recording's `isrcs` holds more ISRCs than the one searched for (So What has three), so the tally counts only the edited tracks' ISRCs, never the recording's. Many reissues share all of an album's ISRCs (Kind of Blue: 8+ six-track releases cover 6/6), so the fallback picker can show near-equal confidences. The barcode path is what usually tells editions apart.
- Barcode search matches regardless of leading zeros (`0602448513403`, `602448513403` and the stored `00602448513403` all hit), so Qobuz's `upc` is sent as is. Every barcode hit scores 100.
- A release lookup lists every track: 42 of 42 on a single-disc release, with no 25-item cap.
- Recording → `performance` → work → `composer` relations come back with the `inc` list above. On the 42-track release, 32 tracks have a work.
- The release's own `genres` can be empty while its release group's are not (Kind of Blue: none vs. jazz 28, cool jazz 7, …), so the release-group fallback is needed.
- A release can have several labels (Columbia, Legacy). The first `label-info` entry is used.
- Found while testing in the app: Qobuz can sell its own digital edition under a barcode MusicBrainz doesn't have. "Richard Hickox conducts Vaughan Williams" is `0724357398657` on Qobuz, while MusicBrainz has only the CD's `724357398626`. Of its 36 ISRCs, MusicBrainz knows one (The Lark Ascending), on two 147-track "Classical Music for Dummies" boxes. Without the half-of-the-ISRCs rule and the track-count condition on position matching, those boxes were offered and would have filled 35 files from unrelated tracks. The title fallback then finds it: "Hickox Conducts Vaughan Williams" shares 4 of 5 words and has the album's 36 tracks on 2 discs (GB 83 %, XE 70 % live).
- In release search, `tracks:36` does not match the Hickox release (2 × 18). The field counts one medium's tracks, not the release's, so the title search constrains only `mediums:` in the query and checks the release's `track-count` on the results. Requiring the album's exact track count is what keeps position matching safe on a release found by title, which has no ISRCs to match by. Releases that only share a word such as "conducts" are dropped by the title-likeness threshold ("Rattle Conducts Britten", also 36 tracks on 2 discs).
- Qobuz `album/get` carries `upc` as a string, per the [minim Qobuz API reference](https://minim.readthedocs.io/en/dev/api-ref/minim.api.qobuz.PrivateAlbumsAPI.html). Confirmed live while testing in the app: the lookup logged Qobuz's barcode `0724357398657` for "Richard Hickox conducts Vaughan Williams".

### Confidence
Computed in `qobuz-core` from the full release and from the album on disk (edited tracks' ISRCs, Qobuz `tracks_count`, title and label from the queue job):

| Signal | Weight | Value |
|---|---|---|
| ISRC coverage | 60 | share of edited tracks with an ISRC whose ISRC is on the release |
| Track count | 20 | 1 when equal, otherwise `max(0, 1 − |Δ| / ours)` |
| Title | 10 | share of words the two titles have in common (shared ÷ all), ignoring case and punctuation |
| Label | 5 | label names equal after normalizing; left out when MusicBrainz lists no label |
| Status | 5 | release is Official; left out when MusicBrainz gives no status |

The result is rounded to a whole percent. When no edited track has an ISRC, or MusicBrainz lists none for the release, the ISRC weight is spread over the other signals in proportion, so a release still ranks: MusicBrainz often has no ISRCs for a CD-era compilation, and scoring that as a total miss put the right Hickox release at 30 %. MusicBrainz's own search `score` is not used: for an exact barcode query it is 100 for every hit.

### Mapping a release onto `TagFields`
A pure function returns one `Option<TagFields>` for each edited track (`None` when unmatched):
- **Matching:** the file's ISRC is among the release track's recording ISRCs; otherwise, only when the release's track count equals the album's, the file's disc and track number equal the medium and track position. Positions on a release laid out otherwise (a box set holding one recording) would fill files from unrelated tracks.
- **Album artist / artist:** the artist credit, each name followed by its join phrase ("Miles Davis feat. …").
- **Date:** `release-group.first-release-date`; when it is empty, the release's `date`.
- **Genre:** the release's genre with the most votes; when there is none, the release group's; ties broken alphabetically; spelled as given.
- **Label:** the first `label-info` label name.
- **Disc total:** the number of audio media; DVD-Video, VHS, data discs and the like are not discs of the album (Blu-ray and plain DVD count, since both also carry audio-only releases). **Track total:** that track's medium `track-count`, per disc, as the editor already treats it. The release's own track count, used for matching by position and for confidence, likewise counts audio media only, or takes a search result's release-wide `track-count`, since a search lists only the medium holding the matched recording.
- **Compilation:** on when the release group's secondary types include Compilation; otherwise set to off (empty), the way `EditorTrack::qobuz` writes a clean track's explicit flag.
- **Composer:** the composer relations of the works the recording performs, deduplicated, in order, joined with `"; "`.
- ISRC, explicit, copyright and comment are never in the result.

### One merge for both sources
`reset_to_qobuz`'s body becomes `TagEditor::fill(per_track: Vec<Option<TagFields>>)`. Reset calls it with `Some(qobuz())` for every track, and MusicBrainz with the mapping's result. An album field is set when all matched tracks agree. A track field is set for matched tracks only. This is the second real caller, so the shared helper is justified, and Reset's tests keep guarding the merge.

### Editor state
`TagEditor` gains `include_cover: bool` (off on open) and `lookup: Option<Lookup>`:
- `Searching(Option<Step>)`: the lookup runs, at this step once one is reached (barcode, ISRCs, title, reading release n of m).
- `Choosing(Vec<Candidate>)`: several releases were found. This is what makes the view draw the picker in place of the track list.
- `Cover(Box<Candidate>)`: a release was chosen with Include cover on. Nothing is filled until the cover arrives, so a failed download changes no field.

`busy()` is `saving.is_some() || lookup.is_some()`. It disables Save, Reset, Fill, the cover buttons and every editor input, so nothing typed or picked during a lookup is overwritten when the fill lands. Results ("Filled from MusicBrainz: Kind of Blue (2003, US) · 96 % match", "1 track not matched") go to the app's `Status` line, and "no release matches this album" is an error there.

A release matching fewer than half the edited files is likely another edition. `fill_from` then fills only the matched tracks and leaves the album fields and cover alone, and the status says so.

### Async bridge and cancellation
The lookup runs as a stream of `tasks::LookupEvent`s (`Step(Step)`, then `Found(Result<Vec<Candidate>, String>)`), bridged with `iced::stream::channel` the same way download progress is, because a lookup can take several seconds. The stream and the cover download are each started with `Task::abortable`. The handle is kept on `App.musicbrainz`, not in the editor: `iced::task::Handle` is `Clone`, and with `abort_on_drop` a dropped clone of the editor would abort the lookup. So the app calls `abort()` explicitly:
- on `Message::CancelLookup`, sent by the Fill button (which reads Cancel lookup while a lookup runs) or the picker's Cancel, at any step;
- when the editor closes.

The handle is cleared when the lookup ends. Each event carries the album id, and an event is dropped unless that album's editor is still searching, so a late result can neither fill nor report anything.

### Cover
After a release is chosen with Include cover on, `musicbrainz::front_cover` fetches `https://coverartarchive.org/release/<mbid>/front-1200` with the MusicBrainz client's User-Agent (reqwest follows the redirect to archive.org). On a 404 it asks for `front` once, and a 404 there means no front cover. `download::fetch_bytes` is not used: it can't tell a 404 from a network error. The bytes go through the same `readable_cover` check as a picked file, then `replace_cover`. The 1200 px thumbnail is preferred, because originals can be scans of tens of megabytes, and every resize option is 600 px or below anyway.

## Risks / Trade-offs

- [MusicBrainz data is wrong or incomplete] → It only fills the form. The user sees every value before Save, and a field with no value is left alone.
- [An ISRC belongs to several recordings, or is reused across editions] → Matching is by ISRC *within the chosen release*, so a reused ISRC only matters when picking candidates. That's what the confidence and the picker are for.
- [Not every recording has a work] → Composer is filled only for tracks whose recording has one (32 of 42 on the long fixture). The others keep their composer, as the spec's "no value, left alone" rule says.
- [Some Qobuz albums have no or wrong `upc`] → The ISRC fallback covers it with one search request. Its candidates can tie, so the picker is shown.
- [The ISRC search response is large (about 1 MB for a much-reissued album)] → It is one request, parsed once; only release ids, track counts and ISRCs are kept.
- [A `front-1200` thumbnail is missing] → The Cover Art Archive documents that a thumbnail "may resolve to a 404 if the thumbnail does not exist", so a 404 on `front-1200` is retried once on `front`. A 404 there means no front cover, which the spike saw on a release without art.
- [The 1 request/s limit is shared with other MusicBrainz apps on the same IP] → `with_retry` backs off on 503.

## Migration Plan

None. No stored data or settings change. Network access to MusicBrainz happens only when the user asks for a fill. To roll back, revert the change.
