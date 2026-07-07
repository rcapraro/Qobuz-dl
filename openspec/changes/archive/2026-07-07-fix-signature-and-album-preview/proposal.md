## Why

Downloads currently fail for every track with `HTTP 400: Invalid Request
Signature parameter (request_sig)`. Root cause: `QobuzClient::get` classifies
signature failures with a **case-sensitive** substring check
(`text.contains("invalid")`), but the API returns "**I**nvalid Request
Signature…". So the error is returned as a plain `Http { status: 400 }` instead
of `Error::InvalidSignature`, and `file_url`'s multi-candidate-secret retry —
which only advances on `Error::InvalidSignature` — never tries the other
auto-detected secrets. The first (wrong) candidate's 400 is surfaced and the
download dies, even though a valid secret is among the candidates.

Separately, search results are text-only; adding **album cover thumbnails**
makes results far easier to scan and pick.

## What Changes

- Fix signature-error detection in `QobuzClient::get` to be case-insensitive (and
  also recognize `request_sig`), so signed file-URL requests correctly fall
  through to the next candidate secret until one is accepted.
- Show an **album cover thumbnail** next to each album in search results,
  loaded asynchronously and cached, with a graceful placeholder while loading or
  if unavailable.

## Capabilities

### New Capabilities
<!-- none -->

### Modified Capabilities
- `audio-download`: the signed file-URL request SHALL recover from a rejected
  signature by trying the remaining candidate secrets (fixing the mis-classified
  error that broke that recovery).
- `downloader-gui`: the search-and-add screen SHALL display album cover
  thumbnails for album results.

## Impact

- **qobuz-core** (`client.rs`): case-insensitive signature-error detection in
  `get`. No API change. This alone restores downloads.
- **qobuz-gui** (`app.rs`): album search results carry a cover URL; new async
  thumbnail loading + in-memory cache; album rows render a small cover image.
  Uses the already-enabled `iced` `image` feature.
- **Models**: `Album.image` already exists — no core model change.
- No new dependencies.
