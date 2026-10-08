# Proposal

## Why

Embedded cover art is always Qobuz's 600px `large` rendition, embedded as downloaded and repeated in every track of an album. A user who wants smaller files has no way to shrink it. Downloads also miss common tags that Qobuz provides: the explicit flag (which `file-organization` already promises but the code never writes), total tracks, total discs, label, copyright and the full release date.

This change is the base for `album-tag-editor` and `rename-album-folder`, decided on 2026-10-08: the editor edits the same field list and replaces covers through the same resizing pipeline.

## What Changes

- **Cover art setting:** the "Embed cover art" checkbox becomes one Cover art selector: Off, 400 px, 500 px or 600 px (Qobuz's own size, explained in the Options help). Off and 600 px match today's two behaviors. The default is 600 px, so existing users see no change until they pick another size.
- **High-quality downscaling:** at 400 px or 500 px, Qobuz's 600 px cover is downscaled with a Lanczos3 filter in linear light and re-encoded as a high-quality JPEG without chroma subsampling. A cover that already fits is embedded byte for byte. Art is never upscaled. The same routine later resizes covers already embedded in files, from the tag editor (`album-tag-editor`).
- **More tags on download:** explicit flag, total tracks (per disc), total discs, label, copyright and the full release date are written alongside the existing tags, in FLAC, MP3 and M4A.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `file-organization`: "Write audio tags" lists the new fields, "Embed cover art" depends on the Cover art setting, and new requirements define the size cap and the downscaling quality.
- `app-configuration`: "Persist settings" includes the cover art size.
- `downloader-gui`: "Settings screen" and "Single-line Options card" replace the cover-art toggle with the Cover art selector.
- `settings-help`: "Options help" explains the Cover art selector.

## Impact

- `crates/qobuz-core/src/config.rs`: a cover size setting next to `embed_art`, loaded with a default for older configs.
- New `crates/qobuz-core/src/artwork.rs`: the cover sizes, and deciding whether to resize, resampling and encoding.
- `crates/qobuz-core/src/engine.rs`: `fetch_cover` processes the cover on a blocking thread before caching it per album.
- `crates/qobuz-core/src/tagging.rs` and `models.rs`: the new fields, including the album's `copyright` and the per-disc track count.
- `crates/qobuz-gui/src/app/view/settings.rs` and `app/help.rs`: the selector and its help.
- Dependencies: `image` (already in `Cargo.lock` through iced, at 0.24.9) added to `qobuz-core` with JPEG and PNG decoding only, and `jpeg-encoder` 0.7 for 4:4:4 encoding.
