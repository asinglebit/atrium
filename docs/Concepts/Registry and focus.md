# Registry and focus

`src/core/registry.rs`. The set of held [[Agents]] and which one the [[Stage]]
is showing — a `Vec<Agent>` and an index into it.

## Moving the focus

| Call | Behaviour |
| --- | --- |
| `push` | A newly held agent takes the stage, which is what you want every time |
| `focus_next` / `focus_prev` | Wrap in both directions |
| `focus_at` | Out-of-range jumps are **ignored rather than clamped**, so a stray digit never moves the stage somewhere you did not ask for |

## Closing one

`dismiss_at(index)` removes that row and returns the `Agent`, whose drop is what
kills it. Two details:

- A row removed **above** the focused one shifts it down, so the focus decrements
  to follow the agent it was on rather than sliding onto a neighbour.
- The focus is then clamped, so it always points at something while anything is
  left.

`dismiss_focused` is just `dismiss_at(focus)`. Dropping the last agent leaves
the registry empty, which is not the end: atrium falls back to the [[Splash]]
and waits to be told what to hold next. `quit` is how you leave.

## Work done every frame

- **`resize_all`** resizes *every* agent, not only the focused one. A background
  agent whose pty still thinks it is the old size renders wrong for a frame when
  you switch to it; resizing all of them costs nothing and removes the flicker.
- **`refresh`** asks each session whether its child is still alive, which is the
  only status change atrium can see on its own.
- **`apply(report)`** routes an incoming [[Adapters and hooks|hook report]] by
  agent id, so reports land correctly even after rows have been reordered or
  closed.

## Work done on a timer

`refresh_git` re-reads the branch and dirty flag for every agent, every **three
seconds** — never in the draw loop. `git status` on a large repository is far
too slow to run at frame rate. Three seconds is slow enough to be free and fast
enough that the `*` appears while you still care.

Dirtiness counts untracked files but does not recurse into untracked
directories: walking them does not change the answer and costs the most time.
A detached head shows a short hash, and a repository with no commits yet says
`unborn` rather than looking broken.
