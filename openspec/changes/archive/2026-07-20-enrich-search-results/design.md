## Context

The Search/Add screen (`crates/qobuz-gui/src/app/view/search.rs`) shows each
result as a single label built in `app/tasks.rs::do_search`
(`"{artist} — {title}"`) plus, for albums, a cover thumbnail. The display
models (`AlbumResult { id, label, cover }` and `(id, label)` tuples for tracks
and artists in `app.rs`) discard everything else the core returns.

`qobuz-core`'s search models (`Album`, `Track` in `models.rs`) do not
deserialize the `hires` / `hires_streamable` booleans that Qobuz's
`catalog/search` response includes — unknown JSON fields are dropped by
design. Quality is otherwise only known post-download, from the signed
`FileUrl` response `format_id`.

The GUI never touches HTTP/IO directly; it consumes core models via
`do_search`. The change is therefore a display enrichment plus a small
model addition — no new endpoint, no request signing, no auth change.

## Goals / Non-Goals

**Goals:**
- Surface a per-item hi-res indicator from the existing search response.
- Show a "Hi-Res" badge on hi-res album/track rows, reusing the existing
  `iced_aw` badge pattern from the queue screen.
- Render album/track rows with title and artist on separate lines.

**Non-Goals:**
- No quality *filtering* of search results (no "hi-res only" toggle).
- No "new releases" browsing or date filtering.
- No exact bit-depth/sample-rate display — a hi-res/not badge is sufficient.
- No change to artist result rows (single name, no quality).
- No new endpoints, signing, auth, download-engine, or config changes.

## Decisions

- **Hi-res source & derivation.** Add `#[serde(default)] hires: bool` and
  `#[serde(default)] hires_streamable: bool` to `Album` and `Track`, plus
  `is_hires(&self) -> bool { self.hires_streamable || self.hires }`. Prefer
  `hires_streamable` (what is actually deliverable) and fall back to `hires`.
  `#[serde(default)]` keeps existing deserialization and tests intact.
- **Display models carry structured fields, not a joined string.** Change
  `AlbumResult` to `{ id, title, artist, cover, hires }`; introduce
  `TrackResult { id, title, artist, hires }` and make
  `SearchPayload.tracks: Vec<TrackResult>`. `do_search` populates these from
  `album.title` / `album.artist_name()` / `album.is_hires()` (and the track
  equivalents). Artists remain `(id, name)`.
- **Row layout mirrors the queue row.** Refactor `add_row` to accept a title,
  an optional artist subtitle, and a `hires: bool`. The middle (fill) element
  becomes a two-line `column![ bold(title), text(artist).size(TEXT_SM) ]`;
  a trailing "Hi-Res" `Badge` renders before the existing
  `secondary_button("Add", …)` when `hires` is true.
- **Reuse the existing badge machinery.** Build the badge exactly like
  `app/view/queue.rs:120` — `Badge::new(text("Hi-Res").size(style::TEXT_SM))`
  with `.style(move |_,_| style::badge(bg, fg))`, taking an accent pair from
  `style::accents(theme)` (a distinct colour from queue status, e.g. teal).

## Risks / Trade-offs

- **Field-name drift:** Qobuz could rename the hi-res flags across web-player
  releases. Mitigated by `#[serde(default)]` (missing → `false`, no crash) and
  by reading both `hires` and `hires_streamable`.
- **`hires_streamable` semantics:** it reflects streamability, which is the
  best available proxy for deliverable hi-res; a rare album flagged `hires`
  but not `hires_streamable` still shows the badge via the fallback.
- **Layout churn:** `add_row`'s signature change touches all three row
  builders; contained to one file and covered by manual GUI verification.
