# Mouse

The mouse is captured, so atrium sees every event — including the ones the
terminal would otherwise have handled itself, like its own right-click menu.

## In the sidebar

| Event | What it does |
| --- | --- |
| Click a row | Puts that agent on the [[Stage]] |
| Wheel | Scrolls the list |
| Drag the last column | Resizes the [[Sidebar]] |

The sidebar's last column is the divider *and* the scrollbar, so that is what
there is to grab. A drag that began there **keeps the mouse until the button is
let go**, however far outside the sidebar the cursor has wandered — otherwise
the resize would stop the moment you dragged past the edge you were dragging
towards.

The width is clamped as you drag, so the agent never drops below 40 columns and
the sidebar never below 16. It is written to `layout.json` **when the drag
settles**, not on every frame of it, so one resize is one write. See
[[Configuration]].

## Anywhere

**Right-click opens the [[Context menu]]**, which is the only way to reach an
action with the mouse.

## In the settings view

Click a tab to open it, click a row to select it, wheel to move — see
[[Settings]].

## In the stage

What the agent gets is what a terminal would give it: reports **only if it asked
for them**. A CLI that never turned mouse reporting on is sent none — a real
terminal sends it none either, and the bytes would arrive in its input as text.
`claude` is one of those: it asks for bracketed paste and hides the cursor, and
never asks for the mouse.

So over the stage the wheel is atrium's to answer. It moves the view through the
history the parser keeps above the agent's screen — 10 000 rows of it, per agent
— three rows a notch, which is what a terminal scrolls for the same notch.
Anything sent to the agent puts the live screen back, the way a terminal drops
you to the bottom the moment you type.

An agent that **did** ask for the mouse gets every event re-encoded instead, its
own wheel included, because it has a use for it that atrium cannot second-guess.
Coordinates are translated to be **relative to the stage's own corner**, so a
click on the stage's first column is the agent's column 1, not column 46. A
click left of the stage is not the agent's and is not forwarded.

The encoding is SGR — `ESC [ < button ; col ; row M` for a press and `m` for a
release, with buttons numbered the way terminals number them.

A **full-screen** agent — one that switched to the alternate screen — has no
history above its screen to move through, so its wheel does nothing unless it
asked for the mouse. That is the same trade a terminal makes.
