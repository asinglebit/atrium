# Sidebar

One row per held agent, down the left. `src/app/draw/sidebar.rs`.

```
 ⠙ 1 atrium                         master* │
 ● 2 guitar                            main │
 ○ 3 bazzite                           sway │
```

## A row

`<status mark> <jump number> <name> <profile> <branch>`

- The **mark** is the [[Status]] glyph, spinning when the agent is working.
- The **number** is only on the first nine rows, because only those can be
  jumped to by digit from the goto list (see [[Modals]]).
- The **name** is the working directory's own name.
- The **profile** is which [[Profiles|subscription]] it is held under — absent
  for a bare CLI, since every row would otherwise carry the same word. Two
  agents on one project are told apart by this and nothing else.
- The **branch** carries a `*` when the tree is dirty.

The profile and the branch each get **one column across every row**, sized to
the longest of them, so the names line up on the left and the rest line up on
the right instead of each row finding its own edge. A column no row fills costs
nothing, separating space included. A name is never squeezed below 8 columns to
make room, and the branch gives ground before the profile does — the profile is
what makes two otherwise identical rows readable.

Rows are zebra-striped the way guitar stripes its panes: the focused row takes
`COLOR_GREY_800`, every other row `COLOR_GREY_900`. There is **no heading** —
the title would cost the top row, and the status line already counts what is
held as `2/3`.

## The line between the panes

The sidebar's last column is both the divider and the scrollbar. guitar draws
one the same way: the border glyph `│` is the track and `▌` is the thumb, so one
column does both jobs.

ratatui draws *nothing at all* for a scrollbar whose content length is zero, so
a list that fits would lose the separator entirely. When there is nothing to
scroll, a plain bordered block is drawn in its place.

## Width

| | Columns |
| --- | --- |
| Default | **45** — guitar's `LAYOUT_WIDTH_LEFT_PANE`, so the two open at the same proportions |
| Minimum | **16** — guitar's `LAYOUT_WIDTH_MIN_SIDE_PANE` |
| Left for the agent | never less than **40** |

A terminal too narrow for 45 gets a **squeezed** sidebar rather than none at
all; it is only dropped entirely below 56 columns of interior. Drag the divider
to set it — see [[Mouse]] — and the width is remembered in `layout.json`, see
[[Configuration]].

**It starts hidden.** One agent is the common case, and a row listing it alone
says nothing the status line does not. `ctrl+space` `shift+1` brings it in, at whatever width
you last dragged it to.

The [[Settings]] view and the [[Splash]] hide it for as long as they are up,
without changing what that key last said — neither has rows to put beside it.
