# Layout

Where everything goes. `src/app/state/layout.rs`. Same shape as guitar's.

```
  title left                               title right   <- 1 row
╭──────────────────────────────────────────────────────╮
│ sidebar          │ stage                             │
╰──────────────────────────────────────────────────────╯
  status left                             status right  <- 1 row
```

- A **title line** above and a **status line** below, the rounded frame taking
  what is left. The status line is dropped on a frame under 3 rows tall.
- Both lines split **70/30** between their left and right halves.
- The frame's own border is not floor space: the panes sit inside it.
- A pane draws only the one edge that separates it from the next — the frame
  already supplies the rest — so the line between the [[Sidebar]] and the
  [[Stage]] is single rather than doubled.

## The widths

| Constant | Columns | What it is |
| --- | --- | --- |
| `SIDEBAR_WIDTH` | 45 | guitar's `LAYOUT_WIDTH_LEFT_PANE` |
| `MIN_SIDEBAR_WIDTH` | 16 | guitar's `LAYOUT_WIDTH_MIN_SIDE_PANE` |
| `MIN_STAGE_WIDTH` | 40 | Below this the agent has nowhere to work |

`clamp_sidebar(width, inner)` mirrors guitar's `side_pane_width`: the requested
width, never below 16, never leaving the agent under 40. Below **56 columns of
interior** neither minimum fits and the sidebar is dropped entirely.

This is why the default could move from 28 to 45 without stranding a narrow
terminal: a fixed 45 would have needed 87 columns before showing a sidebar at
all, whereas a clamped 45 squeezes instead.

## The title and status lines

The title line carries `atriuɱ` — the wordmark at its smallest — in
`COLOR_PINK`, then the focused agent's
working directory, truncated **from the front** — the last components are the
ones that identify a path. The right half says what you are looking at:
`agents`, `settings`, `new agent`, `go to`.

The status line carries the focused agent's name, its [[Status]] as a word, and
its branch with a `*` when dirty. The right half is `<focus>/<held>`.

## Helpers

`centered`, `stack_header` and `stack_footer` do the small geometry the modals
and the [[Settings]] view need. A box too big for its frame is shrunk to fit
rather than allowed to overflow.
