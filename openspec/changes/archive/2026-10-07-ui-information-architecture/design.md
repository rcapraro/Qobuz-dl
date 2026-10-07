# Design

## Context

All of this change is in `qobuz-gui`. The relevant current state:

- `App.status` is a plain `String`, written in about 40 places in `App::update`
  and rendered by one `container` in `App::view`.
- `App::new` starts on `Screen::Settings`. The `Tabs` widget pushes Settings,
  Search, and Queue in that order, with static `TabLabel::Text` labels.
- `catalog::parse_input` treats any single ASCII-alphanumeric word (other than
  a kind keyword) as a bare album ID. So "Radiohead" and "1989" parse
  successfully. A single field cannot use "does it parse?" to choose between
  add and search.
- `style.rs` already holds the spacing, typography, palette (`Accents`), and
  builders. `queue.rs` bypasses the builders for five buttons, and card headers
  take an arbitrary accent passed by each caller.

## Goals / Non-Goals

**Goals:**
- Every behavior in the spec deltas is decided in pure functions that can be
  unit-tested without rendering (input classification, remaining count, setup
  state).
- No public API change in `qobuz-core`.

**Non-Goals:**
- Auto-dismissing status messages or toast stacking. A single status line
  stays, and it is typed.
- Replacing tabs with a sidebar.
- Any change to search pagination, album detail, or queue grouping. Those are
  the follow-up changes.

## Decisions

### Omnibox classification by input shape, not by parse success

A pure `classify(input) -> Submit` returns one of:
- `Add(Reference)`: the input looks like a URL (has an `http://` or `https://`
  scheme, or contains `qobuz.com`) and `parse_input` accepts it.
- `BadUrl(error)`: it looks like a URL but `parse_input` rejects it. This is
  reported as an error and not searched, because searching for URL text never
  helps.
- `Search { query, bare_id: Option<Reference> }`: everything else. `bare_id`
  is `parse_input`'s result when it succeeds, and it is rendered as an
  "Add as ID" row above the results.

*Alternative:* tighten `is_bare_id` in core (for example, require digits or a
minimum length). Rejected because Qobuz album IDs include short alphanumeric
hashes, so any heuristic either misses real IDs or still catches words. Offering
the ID explicitly is correct for every input. `url_input`, `Message::UrlChanged`,
and `Message::AddUrl` are removed. `search_query` becomes the only field.

### Typed status as a small enum

```rust
enum StatusKind { Info, Progress, Success, Error }
struct Status { kind: StatusKind, text: String }
```

`App.status` becomes `Option<Status>`, where `None` is the dismissed state. A
`Message::DismissStatus` clears it, and the dismiss control is rendered only
for `Error`. Each `self.status = …` site picks a kind. Constructors
(`Status::info(..)`, `::progress`, `::success`, `::error`) keep the call sites
one line long. The status bar renders an icon glyph and an accent outline
by kind:

| kind | accent | glyph |
|---|---|---|
| Info | `surface2` (neutral) | `•` |
| Progress | yellow | `…` |
| Success | green | `✓` |
| Error | red | `✗` |

All four glyphs are present in the bundled Inter Regular (checked with
`fc-query`), so none relies on OS font fallback. `ℹ` and `☾` are absent, which
rules them out. An outline is used instead of a left stripe because an iced
container border is uniform on all sides. The bar has a fixed height, so a
dismissed message or the dismiss control never shifts the tabs below.

*Alternative:* derive the kind by matching the message text. Rejected because
it is fragile and invisible at the call site.

### Semantic accents as named helpers on `Accents`

`Accents` gains role methods (`brand()`, `primary()`, `success()`,
`progress()`, `error()`, `quality()`) returning the matching flavor color.
Views call roles, not hues, so the mapping lives in one place. `card` no longer
takes a head color. The header uses `surface1` with `text`, and `card` and
`card_el` drop their `head` parameter. Queue status badges map to roles:
queued is neutral, downloading and tagging are progress, done is success, and
error is error. Tagging moves from its own yellow to progress (yellow), so the
two in-flight states share a meaning.

The done badge currently reads `done · FLAC` in green. To give quality its own
accent, it splits into a green `done` badge and a teal quality badge.

### Compact button variant

`style.rs` gains `compact_button(label, msg)` for row actions (Retry, Remove,
and later Add on result rows). It uses `TEXT_SM`, `[SPACE_XS, SPACE_SM]`
padding, a fixed compact height, and no minimum width. Secondary and primary
builders accept `impl text::IntoFragment<'a>` instead of `&'a str`, so owned
labels such as `Retry failed (3)` go through the builder too. That removes the
reason the Queue header hand-built its buttons.

### Queue tab count and launch screen

A pure `remaining(queue) -> usize` counts `Queued | Downloading | Tagging`
items. The tab label is `Queue (N)` when N > 0, otherwise `Queue`. `App::new`
starts on `Screen::Search`. The successful-resolve path still switches to
Queue, as it does today.

### Setup prompt

A pure `setup_gap(&Config, signed_in) -> Option<SetupGap>`, where `SetupGap`
is `Credentials` or `SignIn`. Credentials are checked first, because sign-in
needs them. When it returns `Some`, the Search screen keeps the field and
replaces the results area with a short message and a primary
"Open Settings" button (`Message::Navigate(Screen::Settings)`). The startup
status strings that told the user to visit Settings become redundant and are
replaced by this prompt.

## Risks / Trade-offs

- [The `card` signature change touches every card call site] → This is
  mechanical, and the compiler finds every call site.
- [Users who habitually paste bare IDs now need one more click] → The
  "Add as ID" row appears immediately above the results. Full URLs, the
  common case, still add in one step.
- [Removing the colored card headers may look flatter] → Accent is kept where
  it carries meaning (active tab, primary buttons, badges, status stripe). The
  card border and header surface still separate sections.
- [Splitting `Done(q)` into two badges widens queue rows] → Rows already use
  `Length::Fill` for the title, so the badges take space from the title, which
  wraps onto a second line before the badges are squeezed.

## Migration Plan

No persisted data changes. `Config` is untouched, and the theme preference
still applies. Rollback is a revert.
