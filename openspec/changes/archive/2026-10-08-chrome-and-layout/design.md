# Design

## Context

See proposal.md for motivation and the two delta specs for the requirements. The chrome lives in `App::view` (`app.rs`): a column of header, `view::status_bar` and an iced_aw `Tabs`, padded by `SPACE_XL`. Each tab's view is wrapped by `tab_pane`, a container with `style::panel` (a bordered, rounded `base` surface) and `SPACE_LG` inner padding.

Four scrollables use `gutter_padding()`, which pads `SCROLLBAR_GUTTER` (12px) on both sides:
- the search results,
- the Settings column,
- the queue list,
- the album track table and its column header.

The fixed controls above them have no such inset, so they start 12px further left than the scrolled content. iced_aw is pinned at 0.12.2, whose `tab_bar::Style` has background, border, label background, label border and text/icon colours. It has no per-side label border, so an underline indicator is not available.

## Goals / Non-Goals

**Goals:**
- One left edge for header, status line, tabs and every tab's content.
- Accent colour in the chrome only where it carries meaning: the error outline and the active tab.

**Non-Goals:**
- Changing the status line's lifetime, height or dismiss behaviour.
- Compact or left-aligned tabs. Tabs stay full width with equal segments, since shrinking iced_aw tab labels is untested and not needed for the lighter look.
- Renaming "Search/Add" in other requirements. The spec keeps "Search/Add" as the section's name and only the visible tab label changes, so the other requirements stay valid without being rewritten.

## Decisions

### Status line: outline only for errors
`status_bar` already resolves each kind to an accent role (`status_look`). The container style becomes `status_surface(theme, outline: Option<Color>)`, passed `Some(error)` for `StatusKind::Error` and `None` otherwise. `None` draws the same `surface0` fill with a neutral `surface2` border at the same width, so the bar's size and position never change between kinds. The icon keeps its accent colour for every kind.

*Alternative:* dropping the border entirely for non-errors. Rejected because the `surface0` fill on the `base` background would lose its edge in the light theme, and toggling border width would shift content by a pixel.

### Theme toggle: compact button with the target theme
The header uses `compact_button`, labelled with the theme it switches to ("Light" / "Dark"). A leading ☀/☾ glyph was tried: ☀ renders in the bundled Inter, but ☾ shows as a missing-glyph box, so neither is used. Text alone keeps the toggle self-explanatory.

### Tab bar: no slabs
`style::tab_bar`:
- **Bar:** `background` becomes `None` and `border_width` becomes 0, so the bar sits on the window background.
- **Active tab:** `surface1` label background with a 1.5px `primary()` outline and regular text. Blue still means active selection, per the semantic accent rule.
- **Hovered tab:** the same `surface1` background without the outline.
- **Inactive tab:** transparent background with regular text.

Two constraints rule out the more obvious "blue text on `surface1`":
- **Hover hides the active state.** iced_aw 0.12.2 checks hover before selection (`tab_bar.rs`), so the style function receives `Hovered`, not `Active`, for the selected tab under the pointer and cannot tell the two apart. Sharing the active surface on hover means the selected tab still looks raised; only the outline drops while the pointer is over it.
- **Contrast in Latte.** Blue `#1e66f5` text on `surface1` `#bcc0cc` is about 2.7:1, under the 3:1 minimum for large text. Regular text on `surface1` stays readable in both flavours.

The status line's info icon moves from `surface2` (about 1.4:1 on `surface0` in Latte, and the same colour as the neutral border) to `subtext`. It is now the only cue for an info message's kind.

The label padding and text size are unchanged, so the bar's height is unchanged.

### Unframed tab content, right-only gutter
- `tab_pane` drops `.style(style::panel)` and its inner `.padding(SPACE_LG)`, keeping only `Fill` sizing. `style::panel` has no other user and is removed.
- A `SPACE_LG` gap separates the tab bar from the content, so the content doesn't touch the active tab. It comes either from the `Tabs` content spacing or from a top padding on `tab_pane`, whichever iced_aw 0.12.2 supports.
- `gutter_padding()` keeps its name and becomes right-only.

Every scrolled view therefore starts at the window's `SPACE_XL` content edge, the same as the header and tab bar.

*Alternative:* adding the same 12px inset to the fixed controls instead. Rejected because it keeps an inset that only exists to make room for a scrollbar on the opposite side.

## Risks / Trade-offs

- [Cards now sit directly on the window background] → Cards already have their own border and `surface0` fill, so they keep their edges. The album header's lavender identity panel is unaffected.
- [The right-only gutter puts the scrollbar against the content's right edge] → The scrollbar sits in the 12px gutter it always had; only the unused mirror gutter on the left goes.
- [The ☾ glyph may be missing from Inter] → The label is text first. The glyph is added only after it is checked in the running app, and is left out otherwise.
- [All four README screenshots change] → Refreshed in the workflow follow-up, at the 949px height this display allows.
