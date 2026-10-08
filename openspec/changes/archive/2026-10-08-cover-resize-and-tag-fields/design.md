# Design

## Context

See proposal.md for motivation and the specs for requirements.

`engine::fetch_cover` downloads `Image::best()` (the `large` URL, a `…_600.jpg` rendition on Qobuz's CDN) once per album into `cover_cache`, and `finish_download` embeds those bytes through `tagging::write_tags`. `Config::embed_art` turns embedding on or off. `write_tags` builds a fresh generic lofty `Tag` from `Track` and `Album`. `Album` has no `copyright` field. A `Job` built from a track search result carries the album embedded in that result, without its track list.

Verified against the sources in the local cargo registry and docs.rs on 2026-10-08:
- `image` 0.24.9 is already in `Cargo.lock` through iced. `imageops::FilterType::Lanczos3` exists. `imageops/sample.rs` states that interpolation runs "linearly over the colour values", without gamma correction. Its `JpegEncoder` always uses 4:2:2 subsampling.
- `jpeg-encoder` 0.7.1: `Encoder::new(w, quality: u8)`, `set_sampling_factor(SamplingFactor::R_4_4_4)`, `set_optimized_huffman_tables(bool)`, `set_progressive(bool)`, `encode(self, data, width: u16, height: u16, ColorType::Rgb)`.
- lofty 0.24.0 `ItemKey` mappings: `Label`, `CopyrightMessage`, `TrackTotal`, `DiscTotal` and `FlagCompilation` exist for Vorbis, ID3v2 and MP4. `ParentalAdvisory` maps to ID3v2 `TXXX:ITUNESADVISORY` and MP4 `rtng`, but has no Vorbis mapping.

## Goals / Non-Goals

**Goals:**
- Reduced covers that look as good as a careful manual resize: no darkened fine detail, no color fringing.
- No behavior change for users who don't touch the new setting.

**Non-Goals:**
- Saving a separate `cover.jpg` in the album folder.
- Color management: ICC profiles are ignored and covers are treated as sRGB, as Qobuz serves them.
- The GUI's own thumbnails, which keep using the `thumbnail`/`small` renditions.

## Decisions

### Keep `embed_art`, add `cover_size`
`Config` gains `cover_size: CoverSize` (`Px400`, `Px500`, `Px600`; default `Px600`; labels are plain "400 px" etc., because a longer label pushed the Notify checkbox onto two lines) next to the existing `embed_art: bool`. The Settings selector shows Off when `embed_art` is false, and the size otherwise. Picking Off sets `embed_art = false` and leaves `cover_size` alone.

That makes the migration free: an old config has no `cover_size`, so `#[serde(default)]` gives 600 px, and its `embed_art` still decides Off. That's exactly the scenario in `app-configuration`.

*Alternative:* one `cover_art: Option<CoverSize>` field replacing `embed_art`. Rejected: it needs custom deserialization to read old configs, for no gain.

### Source: Qobuz's `large` cover only
Every size fetches `large`, as today. Qobuz's `large` is 600 px:
- At 600 px it fits, so it's embedded unchanged, byte-identical to the current behavior.
- 400 px and 500 px downscale it.

Larger sizes from the undocumented `_org` rendition were considered and dropped on 2026-10-08: the goal is smaller files, not larger art.

### `artwork.rs`: a pure, blocking `prepare_cover(bytes, size) -> Vec<u8>`
It's also the entry point `album-tag-editor` uses to resize covers already embedded in files.
1. Read the image dimensions. If the longest side ≤ cap, return the input bytes unchanged.
2. Decode with `image`, with decoder limits set (maximum dimensions and allocation) so an oversized or hostile file fails instead of exhausting memory. Confirmed in 0.24.9:
   - `Limits` is `#[non_exhaustive]`-like (private field), so it is built from `Limits::default()` with `max_image_width`, `max_image_height` and `max_alloc` set.
   - `io::Reader::new(Cursor).with_guessed_format()?` then `.limits(l)` and `.decode()`.
   - `.into_dimensions()` reads the size from the header alone, for step 1.
   - `to_rgba32f()` normalizes `u8` to [0, 1].
   - `imageops::resize` is generic over the pixel type, and for `f32` clamps its output to [0, 1].
3. Convert to `Rgb32F`: composite any alpha over white, then apply the exact sRGB→linear transfer function (piecewise: linear segment below 0.04045, power 2.4 above).
4. Resize with `imageops::resize(…, FilterType::Lanczos3)` to the cap on the longest side, keeping the aspect ratio and rounding the other side to the nearest pixel (minimum 1).
5. Convert back with linear→sRGB, clamping to [0, 1] first because Lanczos ringing overshoots, then round to `Rgb8`.
6. Encode with `jpeg-encoder` at quality 92, `R_4_4_4`, optimized Huffman tables, baseline rather than progressive, because some car stereos and older players don't decode progressive embedded art.

Any error returns the input bytes, as `High-quality cover downscaling` requires.

### Choice of downscaling filter: Lanczos3 in linear light
The reductions are mild (600 → 500 is ×0.83, 600 → 400 is ×0.67), and covers often carry small text and thin lines. At these ratios the filter decides whether that detail stays crisp:

| Filter | At ×0.67–×0.83 |
|---|---|
| Nearest | Drops pixels: jagged edges, broken thin lines |
| Bilinear (`Triangle`) | Soft, and aliases at non-integer ratios |
| Catmull-Rom / Mitchell (bicubic) | Good, slightly softer than Lanczos, less ringing |
| **Lanczos3** (windowed sinc, 3 lobes) | Sharpest result without aliasing; the usual choice for high-quality reduction |

Lanczos3's one weakness is a faint halo (ringing) beside very hard edges. At these ratios it's barely visible, and clamping keeps it in range.

Working in linear light matters as much as the filter. Averaging gamma-encoded sRGB values darkens fine bright-on-dark detail, such as white text on a black cover. The checkerboard scenario in the spec pins this down.

The image is downscaled in one pass straight to the target size. That's correct for Lanczos, which widens its kernel by the scale factor; repeated halving is a workaround for filters that don't.

*Alternative:* `fast_image_resize`. It's faster with SIMD, but `image`'s resize at most a few times per batch is fast enough, and `image` is already compiled in.

*Alternative:* `mozjpeg`. Rejected: its C/CMake/NASM build would complicate CI and packaging on three platforms.

*Alternative:* PNG output. Rejected: the goal is smaller files, and a lossless 500 px cover is several times the size of the JPEG.

### Processing runs once per album, off the async runtime
`fetch_cover` runs `prepare_cover` inside `tokio::task::spawn_blocking`. The cache holds a `tokio::sync::OnceCell` per album, so tracks of the same album that start together wait for one fetch and resize instead of each running their own. If the blocking task panics (a decoder bug on a malformed cover), the fetched bytes are embedded unchanged and a warning is logged, as the spec's "Unreadable cover still embedded" scenario requires. `config` is fixed for a batch, so the cache key stays the album id. The fetch-and-process future stays raced against cancellation as today.

### Tag fields
- **Release date:** `release_date_original` (`YYYY-MM-DD`) parsed into lofty's `Timestamp`. The year comes from the first four characters, as before; month (1–12) and day (1–31) are added only when present and valid.
- **Label:** `album.label.name` as `ItemKey::Label`.
- **Copyright:** a new `#[serde(default)] copyright: Option<String>` on `Album`, as `ItemKey::CopyrightMessage`.
- **Total discs:** `media_count` via `set_disk_total`.
- **Total tracks per disc:** computed in `engine::resolve` from the album's track list: the number of tracks with the same `media_number`. It's stored on `Job` as `disc_track_total: Option<u32>`. Without a track list, an album whose `media_count` is exactly 1 falls back to `tracks_count`. Any other album writes no total, including one that omits `media_count`, because the album count would be wrong for each disc of a multi-disc album.
- **Explicit:** written only when the track is explicit. A clean track gets no flag rather than a "clean" value, because Qobuz doesn't distinguish "clean" from "not explicit".
  - ID3v2 and MP4 use `ItemKey::ParentalAdvisory` with value `1`.
  - Vorbis comments: lofty 0.24's `ItemKey` has no custom-key variant, so the generic tag is converted with `VorbisComments::from(tag)`, and `ITUNESADVISORY=1` is inserted on that native tag before saving it.
  - Verified in lofty's `Ilst` merge code: the generic `Tag` → `Ilst` conversion parses `"1"` into a signed-integer `rtng` atom (`AdvisoryRating::Explicit`).

## Risks / Trade-offs

- **Decoding cost** → a 600 px cover decodes in milliseconds, once per album, on a blocking thread. Decoder limits still matter for covers the tag editor reads from files or from disk, which can be any size.
- **Lanczos ringing** around hard edges → kept in range by clamping. It's the standard trade-off for sharpness, and much less visible than the darkening caused by resizing on gamma-encoded values.
- **Another JPEG generation** for resized covers → avoided whenever the source already fits, and quality 92 at 4:4:4 keeps the loss below visibility at these sizes.
- **New dependencies** in `qobuz-core`: `image` with `default-features = false` and `jpeg`/`png` only, reusing the locked 0.24.9, and `jpeg-encoder` (pure Rust, no transitive runtime dependencies). `Cargo.lock` changes in the same commit.

## Migration Plan

None beyond the config default above. Rolling back leaves an unknown `cover_size` field in the config, which older versions ignore.
