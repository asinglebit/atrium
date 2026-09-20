# Stage

The right-hand pane: the focused agent's own terminal, drawn exactly as it drew
itself.

`tui-term` renders the `vt100::Parser` screen that the agent's reader thread is
filling — see [[Agents]]. atrium does not interpret what is on it; the agent
painted those cells and atrium just shows them.

## What reaches the agent

Everything atrium does not claim for itself:

- Keys that are not one of the eight bindings in [[Keys]], re-encoded into the
  bytes a terminal would have sent.
- Pasted text, as a bracketed paste. Without bracketed paste enabled, a paste
  arrives at the agent as a burst of individual keystrokes.
- Mouse events inside the stage, but **only when the agent asked the terminal
  for the mouse** — re-encoded with coordinates relative to the stage's own
  corner, since it has no idea there is a sidebar beside it. An agent that never
  asked is sent none, and its wheel scrolls the history below instead. See
  [[Mouse]].

Input goes only to the focused agent, and only while it can still read it.

## History

The parser keeps 10 000 rows above the screen, and the wheel moves the view
through them three rows a notch — the only way to read them, since the agent
itself is drawing a live screen and has no scrollback of its own to offer.

Drawing it takes nothing extra: `vt100` holds the offset and `tui-term` reads
the screen through it, so scrolling back is a number rather than a second
buffer. It is per agent, so each keeps its own place, and anything typed puts
the live screen back.

## The background

The agent paints its own cells, and writes `Color::Reset` for every cell it
never coloured — which a terminal draws in **its** background, not the theme's.
Left alone, the pane that fills most of the screen would be the one pane
ignoring [[Themes|the theme]].

So after the widget has rendered, atrium walks the stage's cells and replaces
`Reset` backgrounds with the theme's. Three things make that the shape it is:

- **`PseudoTerminal::style()` is not the fix.** tui-term 0.3.4 stores it and
  never reads it; `state::handle` only ever consults `cursor.style`.
- **Painting underneath does not work**, because the widget opens with `Clear`.
  The pass has to run *after* it.
- **Only `Reset` is replaced.** A background the agent set on purpose — a
  selected line, a diff marker — is its own and is left alone. Foregrounds are
  left alone too.

## Sizing

Every agent is resized to the stage, not only the focused one — see
[[Registry and focus]]. The stage is recomputed each frame from the terminal's
current size, so a window resize reaches the children immediately.
