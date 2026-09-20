# Settings

`ctrl+space` `?` opens it, `esc` closes it. It takes the whole inside of the frame,
hiding the [[Sidebar]] for as long as it is open.

This is guitar's settings view, copied rather than imitated: same column width,
same rows, same rhythm, same narrowing. What follows is what that means in
practice.

```
(blank)
 version: ······························· 0.1.0     <- shaded
(blank)
        ▟ the wordmark, pink into purple ▙          <- where guitar's heatmap sits
(blank)
 general   display   profiles   shortcuts           <- tab bar
(blank)
(blank)
 paths:                                             <- heading, COLOR_HIGHLIGHTED
(blank)
 config:            ~/.config/atrium/config.toml    <- shaded
 profiles:         ~/.config/atrium/profiles.json
 theme:              ~/.config/atrium/theme.json    <- shaded
 layout:            ~/.config/atrium/layout.json
```

## The column

Everything lines up to one centred column. Guitar sizes it from its heatmap;
atrium keeps the same **8 columns of margin** and the same **106-column
ceiling** and drops the two terms that only exist because heat cells are two
columns wide:

```
width = (pane width - 1 - 8), capped at 106
```

So it grows with the frame and stops where guitar's stops. A row is
`label`, spaces, `value`, filling the column exactly — which is what makes every
row end at the same place whatever is in it.

## Narrowing

Two things give way, in this order:

- **Values elide** with `...`, not `…`. guitar's `truncate_with_ellipsis`, so a
  long path becomes `/home/rattleworks/....` rather than being cut off.
- **The tab bar collapses** to one `•` per tab once the full labels no longer
  fit, rather than overflowing the column.

```
     config: /home/rattleworks/....

            • • • •
```

## Dividers

There are no rules. A section is always **blank line, heading, blank line**, and
rows are told apart by **alternate shading** — every other row takes
`COLOR_GREY_900`. The selected line takes `COLOR_GREY_800` over whatever it
already was.

## Tabs

| Tab | What it holds |
| --- | --- |
| general | The four files atrium reads or writes, the [[Projects]] root, and the [[Adapters and hooks\|status socket]] |
| display | All 60 [[Themes]], with a radio marker on the one in use. Enter applies and writes it |
| profiles | Adding, renaming and deleting [[Profiles]], and under them what atrium found installed and where |
| shortcuts | Every action with its chord, as [[Keys]] lists them |

`tab`, `left` and `right` switch; `j`/`k` and the arrows move; `enter` picks.
A new tab starts at its top.

## The cursor addresses lines, not rows

Headings and blank lines are lines too. Moving nudges the cursor by one, and the
next draw **snaps** it onto something it can land on: the next one in the
direction it was going, and failing that the nearest by distance.

That is why j/k walks a section evenly instead of appearing to stick on the gap
before a heading. A click is different — it lands only on a line that can be
landed on, because the pointer said exactly where it meant.

## Scrollbar

On the frame's own right border, where guitar puts its own, with the same
`╮` `╯` `│` `▌`. The whole view scrolls, wordmark included.
