# Modals

Only one is ever up, and while it is, it owns the keyboard: nothing reaches the
agent, so a stray keystroke cannot land in a conversation you cannot see.

## New agent — `ctrl+t`

Pick a project, pick a CLI.

- **Type** to filter. The list is **ranked, not just filtered**: matching loosely
  means `gui` finds both `guitar` and `asinglebit.github.io` — the letters really
  are in there, in order. Matches are scored by how far apart the matched
  characters sit, then by how late the match starts, then by name length, so the
  tight one wins. Without that the obvious answer is rarely first.
- **`tab`** cycles the CLI: `claude`, `opencode`, `codex`.
- **`enter`** holds it, **`esc`** closes, arrows move, backspace deletes.
- Typing restarts the selection at the top, because the list underneath it has
  just changed.

A CLI that is not installed is **a message, not the end of atrium**. Picking
`codex` when there is no codex used to return an error that propagated out of
the run loop and took the whole UI down; the failure now stays inside the modal
so another choice can be made. Only the first line of it is shown — a spawn
failure names every directory on `PATH`, and nothing after the first line says
anything the reader needs.

See [[Projects]] for where the list comes from.

## Go to — `ctrl+g`

A numbered list of what is held.

- A **digit** jumps straight to that row. Rows are numbered from one, so `0`
  never names anything, and a number that is not there is ignored.
- `j`/`k` and the arrows move for when a row has scrolled past nine, `enter`
  goes, `esc` closes.

This modal exists because `ctrl+1`..`ctrl+9` cannot be bound at all — see
[[Keys]]. Inside a modal the keyboard is atrium's, so a plain digit means what
it says.
