# Status

What an agent is doing, shown as a glyph in the [[Sidebar]] and as a word in the
status line.

| Status | Glyph | Label | Colour |
| --- | --- | --- | --- |
| `Idle` | `○` | idle | `COLOR_GREY_400` |
| `Working` | *spins* | working | `COLOR_AMBER` |
| `NeedsInput` | `●` | needs you | `COLOR_GREEN` |
| `Error` | `✗` | error | `COLOR_RED` |
| `Exited` | `·` | exited | `COLOR_GREY_600` |

A working agent spins where the others show a steady glyph — six frames at
100ms, derived from elapsed time rather than driven by a thread, because the
draw loop already runs often enough to animate it.

The colours are [[Themes|palette]] entries rather than colours of atrium's own,
so atrium and guitar can never disagree about what red is.

## Where a status comes from

Two sources, declared per CLI by its [[Adapters and hooks|adapter]]:

- **`Hooks`** — the CLI calls back into atrium and says what it is doing.
- **`Heuristic`** — nothing calls back, so atrium can only watch the process.

Claude reports through hooks. `opencode` and `codex` are held but only watched.

## What each hook event means

| Event | Status |
| --- | --- |
| `SessionStart`, `Stop` | `Idle` |
| `UserPromptSubmit` | `Working` |
| `Notification`, `PermissionRequest` | `NeedsInput` |
| `StopFailure` | `Error` |
| `SessionEnd` | `Exited` |

Unknown events are ignored rather than guessed at.

## Exited is terminal

Once an agent has exited it stays exited — a hook that arrives late must not
bring a dead row back to life. `Exited` is also the one status atrium can reach
on its own, by noticing the child is gone.
