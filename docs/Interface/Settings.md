# Settings

`ctrl+s` opens it, `esc` closes it. It takes the whole inside of the frame,
hiding the [[Sidebar]] for as long as it is open.

Laid out the way guitar lays its own out: one centred column that everything
lines up to, with a header block above it. guitar heads its settings with a
contribution heatmap and sizes the column to that; atrium has no commits to
plot, so the **wordmark** takes that place.

```
                  █      █
                  █
            ▄▀▀█ ███ █▄▄ █ █  █ █▀█▀█
            █  █  █  █   █ █  █ █ █ █
            ▀▀▀▀  ▀▀ █   █  ▀▀▀ █ █ █

         version ······················ 0.1.0

          shortcuts   themes

             tab switches · enter picks · esc closes

         keys

         new ·························· ctrl+t
         goto ························· ctrl+g
```

## The logo

Five rows, 25 columns. The top two carry only the dot on the `i` and the
ascender on the `t`; the three below are the x-height every letter shares. The
first two rows take `COLOR_PURPLE` and the rest `COLOR_DURPLE`, splitting the
way guitar splits its own logo across two greens. A column with no room for the
block gets the plain word `atrium` instead.

Purple is atrium's colour: the same `COLOR_PURPLE` paints `atrium` in the title
line.

## Tabs

| Tab | What it lists |
| --- | --- |
| shortcuts | Every action with its chord, as [[Keys]] describes them |
| themes | All 47 presets, with the one in use marked |

`tab`, `left` and `right` switch tabs; `j`/`k` and the arrows move; `enter`
applies the theme under the cursor and writes it — see [[Themes]]. Each tab
keeps its own cursor, so switching back and forth does not lose your place.

## Geometry

The column is the frame's width less 8 columns of margin, capped at **48** — a
settings row stretched across a full-screen terminal is unreadable. Section
headings are left-aligned *inside* that centred column, not centred text.

The whole view scrolls as one, logo included, so the scroll trap counts lines
rather than rows. Each draw records where it put the selectable rows and the tab
bar, which is what a click is measured against — building it twice is what would
let drawing and clicking disagree.
