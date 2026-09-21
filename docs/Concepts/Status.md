# Status

What an agent is doing, shown as a glyph in the [[Sidebar]] and as a word in the
status line.

| Status | Glyph | Label | Colour | |
| --- | --- | --- | --- | --- |
| `Idle` | `○` | idle | `COLOR_GREEN` | |
| `Working` | *spins* | working | `COLOR_ORANGE` | pulses |
| `NeedsInput` | `●` | needs you | `COLOR_BLUE` | pulses |
| `Error` | `✗` | error | `COLOR_RED` | |
| `Exited` | `·` | exited | `COLOR_GREY_600` | |

The two statuses that are still waiting — one on you, one on the model — pulse:
on the dark half of the beat they drop to `COLOR_GREY_600` and come back. The
other three are settled, so nothing about them moves. Green is a finished agent
rather than an absent one, which is why idle is the colour for everything done.

**The row on the stage never pulses**, wherever it is drawn -- the sidebar, the
goto list, the status line. The pulse is there to pull your eye to a row you are
not looking at, and that is the one row you are. It keeps its colour and its
status: the row is still blue, the tmuxbar segment still says `needs-input`, and
glancing at an agent without answering it does not lose the reminder. Only the
flashing stops.

A working agent spins where the others show a steady glyph — six frames at
100ms, derived from elapsed time rather than driven by a thread, because the
draw loop already runs often enough to animate it. Its row therefore both spins
and pulses; the spin says which row, the pulse says the same thing the window
name in tmuxbar is saying.

## The pulse is drawn, not asked for

`Modifier::SLOW_BLINK` is the obvious way to write this and it does nothing: it
emits SGR 5, which ghostty parses and ignores. So the beat is a colour chosen
twice, `spinner::pulse_at`, off the same clock the spinner comes off — except
that it reads the **wall clock** rather than this process' uptime, so two
atriums on one machine light up together instead of each keeping its own time.

That matters because the beat leaves the process: tmuxbar cannot keep time
either, and atrium hands it one. See [[In tmux]].

The colours are [[Themes|palette]] entries rather than colours of atrium's own,
so atrium and guitar can never disagree about what red is. tmuxbar reads the
same palette and paints a window by the same rule, so a glance at the window
list says what a glance at the sidebar says.

## Where a status comes from

Two sources, declared per CLI by its [[Adapters and hooks|adapter]]:

- **`Hooks`** — the CLI calls back into atrium and says what it is doing.
- **`Heuristic`** — nothing calls back, so atrium can only watch the process.

`claude` and `copilot` report through hooks. `opencode` and `codex` are held
but only watched.

## What each hook event means

| Event | Status |
| --- | --- |
| `SessionStart`, `Stop` | `Idle` |
| `UserPromptSubmit` | `Working` |
| `Notification` (narrowed), `PermissionRequest` | `NeedsInput` |
| `PostToolUse`, `PermissionDenied` | lifts `NeedsInput`, nothing else |
| `StopFailure` | `Error` |
| `SessionEnd` | `Exited` |

Unknown events are ignored rather than guessed at.

### Not every Notification means you

Claude rings `Notification` for eleven different things: a permission prompt and
a question of its own, but also "you have not typed in a while", "the turn is
finished", "you signed in", and three about quota. Registered bare, all eleven
read as `NeedsInput` -- so a row turned blue the moment a turn ended and pulsed
there until it was answered, which is exactly the state the colour was supposed
to be distinguishable from.

It is registered with a **matcher** now, naming the four that mean you:
`permission_prompt`, `elicitation_dialog`, `elicitation_url_dialog`,
`agent_needs_input`. The matcher does the telling apart inside claude, so the
handler still takes its event name as an argument and still reads no payload.
See [[Adapters and hooks]].

## Two events that only lift a wait

Claude says nothing at the moment you answer a permission prompt. Left to the
table above, `NeedsInput` would stand from the prompt until the end of the whole
turn — the agent working away under a row that still says it wants you.

The tool going ahead is the first word of it, so `PostToolUse` and
`PermissionDenied` are registered too. They **only** turn `NeedsInput` into
`Working` and leave every other status where it stands, which is what keeps
them honest: hooks are `async` and arrive out of order, and `PostToolUse` fires
on every tool call rather than only the ones that waited on you. One landing
after `Stop` must not pull a finished turn back to working.

## A second spelling, for matching

Inside tmux the same statuses are written onto the pane as one word each, where
`NeedsInput` becomes `needs-input` rather than "needs you" -- the value is
matched against inside a format string, and a space would not survive it. The
labels above are for reading. See [[In tmux]].

## Exited is terminal

Once an agent has exited it stays exited — a hook that arrives late must not
bring a dead row back to life. `Exited` is also the one status atrium can reach
on its own, by noticing the child is gone.
