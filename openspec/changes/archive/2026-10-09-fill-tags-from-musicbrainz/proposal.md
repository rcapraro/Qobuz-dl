# Proposal

## Why

Qobuz metadata is good but uneven: composers are often missing outside classical music, a remaster carries its reissue date rather than the original one, and genres are broad. MusicBrainz fills exactly those gaps. Because every downloaded album has a barcode (UPC) and every track an ISRC, the app can find the matching MusicBrainz release with almost no guesswork, so the tag editor can offer a second source alongside Reset to Qobuz.

## What Changes

- The tag editor gains a **Fill from MusicBrainz** action next to Reset to Qobuz. It looks up the album's MusicBrainz release by barcode, falling back to the tracks' ISRCs and then to its title, and fills the editor's fields from it. As with Reset to Qobuz, nothing is written until the user saves.
- When more than one release matches, the editor's track list is replaced by a **release picker**. It lists each candidate with its date, country, label, format and track count, plus a **match confidence** the app computes from how well the release fits the files on disk, and sorts the candidates by that confidence. A single candidate is used without showing the picker.
- An **Include cover** option, off by default, also replaces the cover with the release's front cover from the Cover Art Archive. The resize option applies to it as it does to any replacement.
- Fields are filled from the release: album, album artist, **original release date** (the release group's first release), the **genre with the most votes**, label, disc and track numbers and totals, compilation, title, artist and **composer** (from the recordings' works). Fields MusicBrainz has no value for (explicit, copyright, comment) are left as they are.
- Requests to MusicBrainz are limited to one per second and identify the app with a User-Agent that carries the app name, its version and the project URL.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `tag-editing`: adds the Fill from MusicBrainz control, the release lookup and its confidence-ranked picker, how fields are filled from a release, the optional cover from the Cover Art Archive, and how failed lookups are reported. Existing requirements keep their behavior.

## Impact

- **qobuz-core**: a new MusicBrainz client module (release search by barcode, release lookup, ISRC lookup, Cover Art Archive fetch, rate limiting, confidence scoring, mapping a release onto `TagFields`). `models::Album` keeps the `upc` Qobuz already returns. No new dependency: `reqwest`, `serde` and `tokio` cover it.
- **qobuz-gui**: tag editor state gains a lookup phase (searching, choosing a release, fetching its cover), cancellable at any step; the editor view shows the picker in place of the track list; a new async wrapper in `app/tasks.rs`.
- **Network**: the app now contacts `musicbrainz.org` and `coverartarchive.org`, but only when the user asks for a fill. It sends the album's barcode, ISRCs, title and disc count, and nothing about the user.
- **Docs**: README's tag editor section and CLAUDE.md's architecture notes.
