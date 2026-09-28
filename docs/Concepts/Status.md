# Status

What an agent is doing, shown as a glyph in the [[Sidebar]], as a word in the
status line, and as the colour of the tmux window it is in.

| Status | Glyph | Label | Colour | |
| --- | --- | --- | --- | --- |
| `Idle` | `○` | idle | `COLOR_GREY_600` | fresh, or stopped by you |
| `Working` | *spins* | working | `COLOR_GREY_600` ↔ `COLOR_GREY_500` | flickers |
| `NeedsInput` | `●` | needs you | `COLOR_ORANGE` | |
| `Done` | `○` | done | `COLOR_GREEN` until seen, then grey | |
| `Error` | `✗` | error | `COLOR_RED` until seen, then grey | |
| `Exited` | `·` | exited | `COLOR_GREY_600` | |

Grey is the colour of nothing to say. A working agent says nothing that needs
you yet, so it stays grey and only flickers a shade lighter; a question is the
one thing that turns a row orange, and it stays orange until it is answered.
How a turn ended -- green for finished, red for failed -- is news only until
you have looked, so it lasts until the agent has been on the stage, and then
the row goes back to grey. The status line is always about the agent on the
stage, so it keeps the status' own colour, seen or not: a failure you are
looking at still reads as one.

`Done` and `Idle` are two statuses because a finished turn is something to say
and a fresh session is not. `/clear` goes back to `Idle`.

**The row on the stage never flickers**, wherever it is drawn -- the sidebar,
the goto list, the status line. The flicker is there to pull your eye to a row
you are not looking at, and that is the one row you are.

A working agent spins where the others show a steady glyph — six frames at
100ms, derived from elapsed time rather than driven by a thread, because the
draw loop already runs often enough to animate it.

## The flicker is drawn, not asked for

`Modifier::SLOW_BLINK` is the obvious way to write this and it does nothing: it
emits SGR 5, which ghostty parses and ignores. So the beat is a colour chosen
twice, `spinner::pulse_at`, off the same clock the spinner comes off — except
that it reads the **wall clock** rather than this process' uptime, so two
atriums on one machine light up together instead of each keeping its own time.

That matters because the beat leaves the process: tmuxbar cannot keep time
either, and atrium hands it one. See [[In tmux]].

The colours are [[Themes|palette]] entries rather than colours of atrium's own,
so atrium and guitar can never disagree about what orange is. tmuxbar reads the
same palette and paints a window by the same rule, so a glance at the window
list says what a glance at the sidebar says.

## Where a status comes from

Every CLI atrium knows reports back, each through whatever it has:

| CLI | Turns begin and end | What happens in between |
| --- | --- | --- |
| `claude` | hooks | its own session file |
| `copilot` | hooks | hooks, and the keys you send it |
| `opencode` | a plugin atrium hands it | the same plugin |
| `codex` | hooks | its window title |

Anything else is held and only watched: atrium can say whether it is alive and
nothing more. See [[Adapters and hooks]] for how each is wired.

## What each word means

Whatever a CLI says, it reaches atrium as one of these words, spelled the way
claude spells its hooks. `Status::after` is the one table that reads them.

| Word | Status |
| --- | --- |
| `SessionStart` | `Idle` |
| `UserPromptSubmit` | `Working` |
| `Notification` (narrowed), `PermissionRequest`, `Elicitation` | `NeedsInput` |
| `PostToolUse`, `PostToolUseFailure`, `PermissionDenied`, `ElicitationResult` | lifts `NeedsInput`, nothing else |
| `Interrupt` | stops `Working` or `NeedsInput`, nothing else |
| `Stop` | `Done` |
| `StopFailure` | `Error` |

Unknown words are ignored rather than guessed at. `SessionEnd` is one of them:
claude fires it on `/clear` and on resume, copilot on `/clear`, and a row taken
for exited stops getting keys -- so an agent that cleared its context went
deaf. Only the child going away means `Exited`.

### Not every Notification means you

Claude rings `Notification` for eleven different things: a permission prompt and
a question of its own, but also "you have not typed in a while", "the turn is
finished", "you signed in", and three about quota. Registered bare, all eleven
read as `NeedsInput`. It is registered with a **matcher** naming the four that
mean you. copilot's `notification` is narrowed the same way, to a permission
prompt and a request for more: registered bare, a background shell finishing
turned its row orange.

## The CLI's own account comes first

Some things no hook reports: claude fires nothing when you approve a prompt,
decline one, or interrupt a turn. Left to hooks, `NeedsInput` stood from the
prompt until the approved tool had *finished* -- minutes, for a build -- and an
interrupted turn went on reading `Working` until the next prompt.

So where a CLI keeps an account of itself, atrium reads it. claude writes
`busy`, `waiting` or `idle` to `<config dir>/sessions/<pid>.json` on every
change; codex, asked to, puts `Working`, `Action Required` or `Ready` in its
window title. Busy and waiting are believed at once. Once that account has been
read, the hooks only say how turns begin and end.

### Idle waits for the hook that says how

Idle looks the same whether a turn finished, failed or was stopped. So the CLI
going idle holds the status where it was for up to 1.5 seconds, for the hook
that tells them apart: `Stop` makes it `Done`, `StopFailure` makes it `Error`,
and nothing arriving means it was stopped, which is `Idle`.

## Order is by stamp, not by arrival

Hooks are separate processes. Each connects whenever it gets round to it, and a
fast turn's `Stop` can land before the `UserPromptSubmit` that started it. So
every input carries a stamp in ms -- `atrium hook` takes one before it does
anything else -- and one stamped earlier than the last applied is dropped. A
hook's stamp is read 100ms early, since it is taken after the moment it
describes; that is what makes a previous turn's late `Stop` lose to the next
turn starting, instead of ending it.

A `Stop` within two seconds of a failure keeps the failure: copilot says that a
turn failed and then that it stopped, and that is one turn failing.

## Where there is nothing to read, the keys

copilot keeps no account of itself, and fires nothing for an answer or an
interrupt. atrium forwards every key you type, so the adapter says what one
means: Esc or Ctrl-C stops a turn, Esc declines a prompt, Enter or a digit
answers one. They are matched on the bytes as sent, so Alt+1 -- `ESC 1` -- is
neither. A tool starting more than a second after a key read as an interrupt
means the key only closed a menu, and the turn goes back to `Working`.

The one this gets wrong: choosing "no" with Enter or a digit reads as answered
until the next prompt.

## A second spelling, for matching

Inside tmux the same statuses are written onto the pane as one word each, where
`NeedsInput` becomes `needs-input` rather than "needs you" -- the value is
matched against inside a format string, and a space would not survive it. The
labels above are for reading. See [[In tmux]].

## Exited is terminal

Once an agent has exited it stays exited — a hook that arrives late must not
bring a dead row back to life. `Exited` is also the one status atrium reaches
only on its own, by noticing the child is gone.
