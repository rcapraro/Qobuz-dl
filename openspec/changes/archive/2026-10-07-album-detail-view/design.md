# Design

## Context

- `engine::resolve(Reference::Album(id))` calls `album/get` and returns one
  `Job { track, album, multi_disc }` per track, in album order. That is
  everything the detail needs: album metadata (`title`, `artist_name()`,
  `year()`, `label`, `genre`, `tracks_count`, `image`, `is_hires()`) and,
  per track, `track_number`, `disc_number()`, `title`, `artist_name()`,
  `duration` (seconds), and `is_hires()`.
- `Message::Resolved(Ok(jobs))` already enqueues jobs, skips tracks already in
  the queue, reports "Added N track(s)", and switches to the Queue tab.
- Search results render in one `scrollable` in `view/search.rs`. Replacing
  that subtree with the detail and back again recreates the widget, so iced
  resets its scroll offset to the top.
- iced 0.13.4: `Scrollable::id(Id)`, `Scrollable::on_scroll(Fn(Viewport))`
  with `Viewport::absolute_offset()`, and `scrollable::scroll_to(Id,
  AbsoluteOffset) -> Task`.

## Goals / Non-Goals

**Goals:**
- No core change and no second fetch: the detail holds the resolved jobs, and
  adding a selection sends a filtered subset of them through the existing
  `Resolved` path.
- Selection and grouping logic in pure functions with unit tests.

**Non-Goals:**
- Opening an album from a track result, or from an artist. Possible later.
- Marking tracks that are already queued or already on disk. `Resolved`
  already de-duplicates against the queue.
- Previews or playback.
- A large cover fetch. The detail reuses the result row's small cover,
  already in the thumbnail cache.

## Decisions

### Detail state lives beside the results, not instead of them

```rust
struct AlbumDetail {
    id: String,
    header: AlbumResult,                 // from the row: shown while loading
    state: DetailState,                  // Loading | Failed(String) | Loaded(Vec<Job>)
    selected: HashSet<i64>,              // track ids; all ids when loaded
}
```

`App.album: Option<AlbumDetail>`. `self.results` is untouched while the
detail is open, so Back simply sets `album = None`. The Search view renders
`album_view` when `album.is_some()`, and results otherwise.

*Alternative:* a fourth `Screen`. Rejected because the detail belongs to the
Search tab: switching to Queue and back should return to the detail, which
`Screen` alone would not express. Tab state already keeps per-tab content.

### Messages

- `OpenAlbum(AlbumResult)`: set `album = Some(Loading)` and run
  `tasks::resolve(Reference::Album(id))` mapped to `AlbumLoaded(id, Result)`.
- `AlbumLoaded(id, Result<Vec<Job>, String>)`: dropped unless `album` is open
  on the same id. This covers Back before load and opening a different album.
- `RetryAlbum`: re-runs the load for the open id.
- `ToggleTrack(i64)`, `SelectAllTracks`, `SelectNoTracks`.
- `AddSelected`: filter the loaded jobs, in album order, to `selected`, then
  reuse `self.update(Message::Resolved(Ok(jobs)))`. The detail stays open, so
  returning to Search lands where the user was.
- `CloseAlbum`: `album = None` plus `scrollable::scroll_to(RESULTS_ID,
  self.results_offset)`.
- `ResultsScrolled(Viewport)`: stores `absolute_offset()` in
  `results_offset`. `SearchSubmit` clears `album` and resets
  `results_offset`, so a new search starts at the top.

### Opening from a row without stealing the Add control

The album row's cover and title/artist block become one borderless button
(text-styled) emitting `OpenAlbum`. The compact "Add" stays a separate button
emitting `Add(Reference::Album)`. Track rows are unchanged.

### Pure helpers

- `discs(jobs) -> Vec<(u32, Vec<&Job>)>`: groups by `disc_number()`, keeping
  album order inside each disc. A single group means no disc headings.
- `selected_jobs(jobs, &selected) -> Vec<Job>`: album order is preserved.
- `format_duration(secs) -> "m:ss"`, plus the album total as `h:mm:ss` when it
  is an hour or longer.
- `guest_performer(job) -> Option<&str>`: the track artist when it differs
  from the album artist.

### Layout

Top to bottom:
- A compact `← Back to results`.
- A header in `style::hero`: a soft wash of a new `Accents::highlight()`
  role (Catppuccin lavender, used only for page identity panels) with a
  matching border. Chosen from a side-by-side comparison of ten candidates
  in both themes. Dropped along the way: mauve (too loud), sapphire (too
  close to the teal quality badge), neutral `crust` (too dark), and
  rosewater. It holds the 160 px cover (the cached small
  image, or the placeholder) beside a headline-size bold title, the artist,
  a muted metadata line (`2019 · Label · Genre · 12 tracks · 48:31`), and a
  Hi-Res badge.
- A selection bar: muted `N of M selected`, compact "Select all" / "Select
  none", and the primary `Add N tracks` at the right.
- The track table on its own surface: a column header in `style::table_head`
  (solid `surface2`, a stronger neutral grey than the stripes, with bold
  labels) so it never reads as one of the striped rows (`#`, `Title`,
  `Time`) on the same grid as the rows, then striped rows (`style::stripe`) of
  checkbox, number, title with a muted guest performer, duration, and a
  Hi-Res badge. Every column but the title is fixed-width, and the badge
  column is reserved even when empty, so the grid holds on every row.
  Unchecked titles are muted. Multi-disc albums get a muted bold "Disc N"
  heading per disc.

Secondary text uses a new `Accents::subtext` (Catppuccin `subtext0`) through
`style::muted_text`, which gives titles the visual weight. Numbers and
durations stay in Inter, right-aligned: iced 0.13 cannot enable Inter's
tabular figures, and monospace digits clashed with the surrounding text.
Every button comes from the `style.rs` builders, and `←` (U+2190) is present
in the bundled Inter.

## Risks / Trade-offs

- [`album/get` may cap the embedded track list on very long albums] → Same
  exposure as adding the album today, since both go through
  `engine::resolve`. The detail shows the album's `tracks_count` beside the
  rows, so a gap would be visible rather than silent.
- [Restoring scroll needs the results to be laid out first] → `scroll_to` is
  a widget operation applied on the next layout pass, after the results
  subtree is rebuilt. If that ordering proves wrong in practice, the fallback
  is to keep the results scrollable mounted and hide it.
- [Holding the jobs in memory] → One album's metadata, dropped on Back.
