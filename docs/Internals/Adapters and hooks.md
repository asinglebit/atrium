# Adapters and hooks

How an agent tells atrium what it is doing. `src/adapters/`, `src/ipc/`.

## Picking an adapter

`detect(program)` matches on the program name with any directories stripped off,
so `/usr/local/bin/claude` and `claude` reach the same adapter.

| Name | Reports via | Notes |
| --- | --- | --- |
| `claude` | Hooks, and its session file | Wears atrium's [[Themes\|theme]], and follows a change while running |
| `copilot` | Hooks, through a plugin atrium generates, and the keys you send it | Keeps its own colours — see [[Themes]] |
| `opencode` | A plugin atrium generates | Wears atrium's [[Themes\|theme]] |
| `codex` | Hooks, through `-c`, and its window title | |
| anything else | Heuristic | Still held; atrium just cannot say more than whether it is alive |

Every agent, recognised or not, carries its identity and the way home:

```
ATRIUM_AGENT_ID=<id>
ATRIUM_SOCK=<path to this atrium's socket>
```

## What an adapter is handed

`instrument` gets the command before it is launched, and a `Wiring`: the way
home (`exe`, `socket`, `agent_id`), the [[Themes|theme]], and the config
directory the profile named, if it named one. The theme is in there because for
claude it travels in the same `--settings` document as the hooks — splitting it
out would have meant a second `--settings`, which replaces rather than adds.

`retheme` is the other half: the theme changed under an agent that is **already
running**. It does nothing by default, and what it is worth differs by CLI —
claude watches its theme file and repaints, opencode reads once and keeps what
it started with. Neither failing to write is treated as an error: an agent in
its own colours is not a broken agent.

An adapter can also say where its CLI keeps an account of itself
(`activity_file`, `read_activity`), what its window title means
(`title_activity`), and what a key sent to it means (`key_meaning`). Each is
nothing by default. See [[Status]] for what atrium does with them.

## Claude's hooks

**Registered through `--settings`, never `~/.claude/settings.json`.** Each agent
is launched with its hooks passed on the command line, so a Claude started
outside atrium is completely unaffected and the global config is never written.

Eleven events are registered, one hook each: `SessionStart`, `UserPromptSubmit`,
`PermissionRequest`, `Elicitation`, `ElicitationResult`, `Notification`,
`PostToolUse`, `PostToolUseFailure`, `PermissionDenied`, `Stop`, `StopFailure`.
See [[Status]] for what each one means.

`SessionEnd` is not among them. It fires on `/clear` and on resume as well as on
the way out, and a row taken for exited stops getting keys.

### Two of them are narrowed

`Notification` is claude's only event that means several unrelated things: a
permission prompt and a question of its own, but also "you have not typed in a
while", "the turn is finished", "you signed in", and three about quota.
`SessionStart` fires for a fresh session, but also after a compaction -- which
can happen in the middle of a turn, and read as fresh it stopped the flicker
while the agent went on working.

Both carry a `matcher` now:

```
"SessionStart":[{"matcher":"startup|resume|clear|fork","hooks":[…]}]
"Notification":[{"matcher":"permission_prompt|elicitation_dialog|elicitation_url_dialog|agent_needs_input","hooks":[…]}]
```

The matcher sits beside `hooks` rather than inside one. Letters, digits, `_`
and `|` are compared by claude as exact alternatives; one character outside
that set would turn the whole thing into an unanchored regex that matched
anything merely containing one of these words, so a test asserts every matcher
stays on the exact-match path.

This is the shape of the thing: the discriminating happens **in claude**, which
is what lets `atrium hook` go on taking its event name as an argument and go on
reading no payload at all.

### The session file says what the hooks cannot

No hook fires when you approve a prompt, decline one, or interrupt a turn. But
claude keeps `<config dir>/sessions/<pid>.json` for itself, and rewrites it on
every change:

```
{"pid":23481, …, "status":"busy", "waitingFor":null, "statusUpdatedAt":1790532030147}
```

`status` is `busy`, `waiting` or `idle`, and `waitingFor` says why when it is
waiting -- except `dialog open`, which is a menu of your own, `/model` or
`/config`, and is read as no news. atrium looks at the file's mtime once a frame
and reads it when that changes.

A read is only believed if its `pid` is this agent's child and its stamp is not
older than the spawn: a file left by a process that had the pid before is
another agent's. Anything it cannot parse -- half written, or a shape a future
claude has moved on to -- is left, and read again next frame.

The first good read hands the file the job of saying what the agent is doing.
From then on the hooks in between are ignored, and only `SessionStart`, `Stop`
and `StopFailure` are heard. If a claude stops writing the file, nothing is
lost: the hooks are all still registered, and say what they said before.

The file is undocumented. It is the same kind of dependency as codex's title,
taken for the same reason: it is the only place the answer is.

### Exec form is a correctness fix, not a preference

Claude's `command` field is a shell string. Written that way, a binary path
containing a quote escapes its own quoting —

```
"/tmp/ev"il" hook Stop
```

— even though the surrounding JSON is perfectly valid. Passing `args` instead
runs the handler directly with no shell, so no path can be re-split or broken
out of. A test asserts this with a path containing both a space and a quote.

Hooks are `async`, so one never sits between Claude and the thing it was doing.

### The event name is an argument, so the handler never parses JSON

Registering one hook per event means `atrium hook Stop` already knows what
happened and can ignore the payload on stdin entirely. That is the whole reason
the handler needs no JSON parser. It still **drains stdin**, to avoid an EPIPE
in the agent that wrote it.

### A hook that fails is a hook that interrupts the agent

`atrium hook` never reports an error: no socket, no environment, a dead atrium —
it exits 0 and says nothing. A missed status update is not worth disturbing a
conversation over.

## Copilot's hooks

**Handed over as `--plugin-dir`, never written into `~/.copilot`.** copilot
loads a plugin from any directory it is pointed at, so atrium generates one and
names it on the command line — the same trade claude's `--settings` makes, and
for the same reason: a copilot started outside atrium is completely unaffected.

The directory is `~/.config/atrium/copilot-plugin/`, with atrium's own files
rather than anywhere copilot reads by itself:

```
plugin.json   the manifest, naming the file beside it
hooks.json    the hooks, grouped by event
```

It is rewritten on every spawn, because the path it carries is wherever this
atrium is installed.

### copilot's event names are translated, not adopted

atrium's wire vocabulary is spelled the way claude spells it, and
`Status::after` is the one table that reads it. So the adapter translates on the
way out rather than teaching `core` a second vocabulary:

| copilot says | narrowed to | atrium hears | which means |
| --- | --- | --- | --- |
| `sessionStart` | | `SessionStart` | idle |
| `userPromptSubmitted` | | `UserPromptSubmit` | working |
| `notification` | `^(permission_prompt\|elicitation_dialog)$` | `Notification` | **needs you** |
| `preToolUse` | `^ask_user$` | `PermissionRequest` | **needs you**: copilot asking a question |
| `preToolUse` | | `PreToolUse` | a turn still going, after a key read as an interrupt |
| `postToolUse`, `postToolUseFailure` | | `PostToolUse` | lifts a wait |
| `agentStop` | | `Stop` | done |
| `errorOccurred` | `"recoverable": false` | `StopFailure` | error |

copilot's matchers are regexes tried against the notification's type or the
tool's name, so each is anchored. `notification` registered bare read a
background shell or agent finishing as needing you.

`errorOccurred` takes no matcher, and copilot reports the errors it gets past
too -- a turn that recovered has not failed. So its entry reads the payload in
the shell, `grep -E` without `-q` so stdin is drained, and calls `atrium hook`
only when the error is not recoverable. It exits 0 either way.

`sessionEnd` is not registered: copilot fires it on `/clear` as well.

### Nothing fires for an answer, so the keys are read

copilot fires nothing when a prompt is answered or a turn stopped. atrium
forwards every key, so the adapter reads the ones that mean something -- see
[[Status]]. Only keys: a paste or a click is not somebody answering a prompt.

### A shell string is quoted, because it cannot be avoided

copilot's hook command is a **shell string**, under a `bash` key. Claude's
`args` escape hatch is not available, so the path is single-quoted and any
quote in it is closed, escaped and reopened — `helpers::shell::quote`. A test
asserts it with a path holding a space, a quote and a `rm -rf` after it.

Only `bash` is written. The status channel is a unix socket, so there is no
Windows to write a `powershell` arm for.

`timeoutSec` is 5. copilot's own default is thirty seconds and there is no
`async` flag to lean on, but the handler writes one line to a socket and exits.

## opencode's plugin

**Handed over as `OPENCODE_CONFIG_CONTENT`, never written into opencode's
config.** opencode merges that variable over your own configuration, so a
config naming one plugin adds it and changes nothing else:

```
OPENCODE_CONFIG_CONTENT={"plugin":["/home/you/.config/atrium/opencode-plugin/atrium.js"]}
```

The path is absolute, the only kind opencode resolves from a config with no file
of its own. If the variable is already set, by you or by a profile, it is left
alone: replacing it would drop everything it says, so that agent goes unwatched
instead.

The plugin is `src/adapters/opencode_plugin.js`, kept as a file of its own so it
reads, and runs, as the JavaScript it is. It is written whole on every spawn. It
listens to opencode's own events and writes atrium's wire lines itself:

| opencode says | atrium hears |
| --- | --- |
| the session you are talking to goes busy, with nothing waiting | `UserPromptSubmit` |
| `permission.asked` or `question.asked`, from any session, with nothing waiting yet | `PermissionRequest` |
| the last of those answered or dismissed | `PostToolUse` |
| the session goes idle | `Stop`, `StopFailure` after an error, `Interrupt` after an abort or a refusal |

A few things it is careful about:
- **One connection, kept open.** Lines on one connection arrive in the order
  they were written, which separate connections would not promise. It is
  `unref`'d, so it never keeps opencode running once it wants to exit.
- **Busy repeats at every step.** Only the first one after idle is a prompt; the
  rest mean an error or a refusal before them did not end the turn.
- **Tasks run in sessions of their own.** Their busy, idle and errors are
  noise, and ignored; their questions are not, since one holds the whole agent
  up.
- **An abort drops what was waiting without a word**, so idle clears it.
- **It forgets how to reach atrium** once it has read `ATRIUM_*`, and takes
  `OPENCODE_CONFIG_CONTENT` out too, so an opencode one of its tools starts does
  not load it again and report as this agent.

## codex's hooks

**Handed over as `-c`, never written into `~/.codex/config.toml`.** `-c` sets a
config key for one process, so three hooks go in -- `UserPromptSubmit`, `Stop`
and `Interrupt`, each a shell string with the path quoted -- and one title:

```
-c hooks.Stop=[{hooks=[{type="command",command="'/path/to/atrium' hook Stop",timeout=5}]}]
-c tui.terminal_title=["activity","run-state"]
```

Each value is TOML, and codex takes one it cannot parse for a plain string, so
a test parses every value back.

Only those three hooks, because they say how a turn began and ended. What
happens in between is in the title codex keeps once asked: `[ ! ] Action
Required` while any approval or question waits on you, a spinner and `Working`
while it works, and `Ready` when the turn is over. The pty parser keeps the last
title, and it is read only when it changes. codex's own `PermissionRequest` hook
also fires for calls it then approves by itself, and nothing fires at all for an
answer; the title knows both.

codex has no error signal. A turn that goes `Ready` with neither `Stop` nor
`Interrupt` behind it settles to idle rather than guessing red.

Two costs: codex asks, the first time, whether to trust hooks it has not seen
-- they are written the same way every time, so it asks once, until atrium runs
from a different path. And a hook of your own in `config.toml` for one of those
three events is replaced for as long as that agent runs.

## The socket

Lives in `$XDG_RUNTIME_DIR/atrium/<pid>-<n>.sock`, falling back to a temp
directory keyed by uid. `XDG_RUNTIME_DIR` is a tmpfs, so nothing survives a
reboot to be mistaken for a live atrium. The counter distinguishes servers
within one process, which the pid alone cannot do.

**Stale sockets are swept on the way in.** An atrium that is `kill`ed rather
than quit never runs its `Drop`, so its socket outlives it. Binding knocks on
each socket it finds and clears the ones nobody answers for, which keeps the
directory from filling up.

Knocking is the whole of the check, and **only a refusal counts**. A connect
that fails for want of a file descriptor, or a permission, says nothing about
the far end — sweeping on that would unlink a living atrium's only door, and
the cost of leaving one stale file behind is that it is swept next time.

This used to read the pid out of the name and look it up in `/proc`. Off Linux
there is no `/proc`, so every pid read as dead and **every** socket was swept,
live ones included: each new atrium unlinked the sockets of the ones already
running, whose agents then reported into a path nothing was listening on. Their
statuses froze wherever they stood, which for an agent that was waiting meant a
[[Status|"needs you"]] that stood for as long as the atrium was up. Knocking
also settles the question the pid never could, since a pid the system has handed
out again reads as alive.

The uid behind the fallback directory is the owner of the home directory, taken
with `MetadataExt::uid`. Not libc, so it still costs no dependency, and not
`/proc/self/status`, which answered `0` for everyone off Linux.

One short-lived thread serves each connection, so a client that connects and
then stalls cannot hold up the ones behind it. opencode's plugin keeps its one
connection for the life of the agent, which holds one thread and nothing else.
The draw loop drains what has arrived without ever blocking.

## The wire format

One line, tab separated:

```
<agent id>\t<event>\t<ms>\n
```

The stamp is when the sender started, in ms since the epoch -- `atrium hook`
takes it before it does anything else. Hooks are separate processes, and they
connect in whatever order they get round to it; the stamp is what puts them
back in the order they happened. It is optional, for a sender that does not
say, and a line without one is taken as arriving now. Malformed lines are
rejected rather than guessed at.

Reports name an agent **by id**, so they land correctly even after rows have
been reordered or closed — see [[Registry and focus]].

## Unix only, deliberately

The status channel is a unix socket. Windows would need a different transport,
and nothing here needs it yet.
