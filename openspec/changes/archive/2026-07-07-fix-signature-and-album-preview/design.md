## Context

Two independent items:

1. **Signature bug.** `QobuzClient::file_url` (client.rs) tries each candidate
   `app_secret` in turn, advancing to the next only when `self.get(...)` returns
   `Error::InvalidSignature`. But `get` decides that via
   `text.contains("invalid") && text.to_lowercase().contains("signature")` — the
   first check is case-sensitive. The live API returns
   `"Invalid Request Signature parameter (request_sig)"` (capital "I"), so the
   condition is false, the error becomes `Error::Http { status: 400 }`, the retry
   loop treats it as fatal (`Err(e) => return Err(e)`), and the first candidate's
   400 is surfaced. Result: 0/21 downloads, all "Invalid Request Signature".

2. **Album thumbnails.** Search results render as text rows. `models::Album`
   already carries `image: Option<Image>` (large/small/thumbnail URLs), so the
   data is available; the GUI just needs to fetch and show it. The `iced` `image`
   feature is already enabled (used for the window icon).

## Goals / Non-Goals

**Goals:**
- Restore downloads by correctly classifying signature rejections so the
  candidate-secret retry works.
- Show a small album cover next to each album search result, loaded async with a
  placeholder and cached so re-renders/re-searches don't re-download.

**Non-Goals:**
- No change to the signing algorithm itself (it is correct; only error
  classification is wrong).
- No thumbnails for tracks/artists in this change (albums only).
- No on-disk thumbnail cache (in-memory per session is enough).

## Decisions

### Decision: Case-insensitive signature detection
In `get`, lowercase once and match robustly:
```rust
let lower = text.to_lowercase();
if lower.contains("signature") && (lower.contains("invalid") || lower.contains("request_sig")) {
    return Err(Error::InvalidSignature);
}
```
This makes `file_url` fall through to the next candidate secret. This one change
restores downloads (the valid secret is already among the auto-detected
candidates).

### Decision: Album search results carry a cover URL
Change `SearchPayload.albums` from `Vec<(String, String)>` to a small struct
`AlbumResult { id: String, label: String, cover: Option<String> }` (or a 3-tuple).
`do_search` fills `cover` from `album.image` preferring `thumbnail`/`small` (a
small size keeps downloads cheap).

### Decision: Async thumbnail loading with an in-memory cache
Add `thumbnails: HashMap<String, image::Handle>` to `App` (keyed by URL). On
`SearchDone`, emit a `Task::batch` of fetches for each album `cover` not already
cached; each resolves to `Message::ThumbnailLoaded(url, Result<Vec<u8>, ()>)`.
On success, insert `image::Handle::from_bytes(bytes)` into the cache. The album
row renders `image(handle)` sized to a fixed thumbnail (e.g. 48–56 px square)
when cached, otherwise a neutral placeholder box of the same size so the layout
doesn't jump. Rationale: results appear instantly; covers stream in; a cache
avoids re-fetching on every `view`/re-search.

Fetching uses a plain `reqwest` GET (images are on Qobuz's CDN, unauthenticated).
A dedicated tiny async helper returns the bytes.

## Risks / Trade-offs

- [A future API message changes wording again] → The broadened check also matches
  `request_sig`, and everything is lower-cased, so it is resilient to case and to
  the "Invalid"/"invalid" split. Cross-referenced to `signature.rs` conventions.
- [Broadened matching misclassifies an unrelated error as a signature failure] →
  Requires the body to contain both "signature" and ("invalid" | "request_sig");
  only `getFileUrl` is signed, so the blast radius is limited to signed calls.
- [Many thumbnail downloads] → Bounded to the ~25 album results per search, fetched
  once and cached; failures fall back to the placeholder and never block the UI.
- [Message carries image bytes] → Bytes are handed straight to `Handle::from_bytes`
  and dropped; only the handles are retained in the cache.
