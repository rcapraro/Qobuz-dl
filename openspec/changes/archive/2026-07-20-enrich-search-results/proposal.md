## Why

Search results render each item as a single concatenated `"artist — title"`
string with no quality information, so users cannot tell hi-res releases from
CD/lossy ones before downloading, and rows are hard to scan. Qobuz's search
response already carries hi-res flags that the app currently discards.

## What Changes

- Surface Qobuz's `hires` / `hires_streamable` flags in the core search
  models (currently dropped as unknown JSON fields).
- Show a small **"Hi-Res"** badge on album and track search rows that are
  hi-res; non-hi-res rows show no badge.
- Restructure each album/track result row to display the **title** (emphasised)
  and the **artist** on a separate secondary line, instead of one flat label.
- Artist result rows are unchanged (a single name, no quality).
- No new endpoints, request signing, or auth changes; the hi-res flag is
  read from the existing `catalog/search` response and filtering by quality
  is explicitly out of scope.

## Capabilities

### New Capabilities

<!-- None: this refines existing search behavior. -->

### Modified Capabilities

- `catalog-browsing`: search results additionally expose a per-item hi-res
  quality indicator derived from the Qobuz search response.
- `downloader-gui`: the Search/Add screen renders album and track results
  with title and artist on separate lines and a Hi-Res badge on hi-res items.

## Impact

- **qobuz-core** — `models.rs`: add `hires` / `hires_streamable` fields
  (`#[serde(default)]`) and an `is_hires()` helper to `Album` and `Track`.
- **qobuz-gui** — `app.rs` display models (`AlbumResult`, new `TrackResult`,
  `SearchPayload`), `app/tasks.rs::do_search` mapping, and
  `app/view/search.rs` row rendering.
- No changes to networking, authentication, download engine, or persisted
  config. Backwards compatible; no breaking changes.
