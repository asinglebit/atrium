# Context menu

**Right-click anywhere.** `src/app/state/menu.rs` decides what it says,
`src/app/draw/menu.rs` draws it.

```
╭─────────────────────────────────╮
│ go to guitar                    │
│ close guitar            ctrl+x  │
│ ─────────────────────────────── │
│ new agent               ctrl+t  │
│ show / hide sidebar     ctrl+o  │
│ settings                ctrl+s  │
│ ─────────────────────────────── │
│ quit                    ctrl+q  │
╰─────────────────────────────────╯
```

## Why right-click

It was a button doing nothing. Mouse capture already suppresses the terminal's
own menu, and the agent ignores a forwarded right-click — so before this there
was no way to reach an action with the mouse at all, and none whatsoever once
the [[Sidebar]] was hidden.

## What it offers

**On an agent's row**, that agent by name first:

- `go to <name>` — put it on the stage
- `close <name>` — end it and take its row away

**Anywhere else** — the [[Stage]], the title line, the status line, empty
sidebar — it opens with `close <focused agent>` instead, when there is one.

**Both** then end with what can always be done: `new agent`,
`show / hide sidebar`, `settings`, and `quit`.

Every entry quotes the chord that does the same thing, so the menu doubles as
the place the [[Keys]] are learnt. The chords come from the live keymap, so a
rebound key shows rebound here.

## Using it

- **Hover** selects, so what a click will do is always the thing under the
  pointer.
- **Click** an entry to pick it; click anywhere off the box to close it. A click
  on the border still counts as a click on the menu.
- **`j`/`k`** and the arrows move, **enter** picks, **esc** closes. Movement
  steps *over* separators rather than stopping on one, and wraps at both ends.
- The wheel moves the cursor too.

While the menu is up it owns the keyboard and the mouse, the same way a modal
does — nothing reaches the agent behind it.

## Where the box lands

At the cursor, pulled back inside the frame when it would otherwise hang off the
right or the bottom, and shrunk if the frame is smaller than the menu. It clears
the cells it covers first, so the agent's screen does not show through it.
