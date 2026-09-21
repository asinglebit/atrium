# Adapters and hooks

How an agent tells atrium what it is doing. `src/adapters/`, `src/ipc/`.

## Picking an adapter

`detect(program)` matches on the program name with any directories stripped off,
so `/usr/local/bin/claude` and `claude` reach the same adapter.

| Name | Reports via | Notes |
| --- | --- | --- |
| `claude` | Hooks | Fully wired. Wears atrium's [[Themes\|theme]], and follows a change while running |
| `copilot` | Hooks | Fully wired, through a plugin atrium generates. Keeps its own colours — see [[Themes]] |
| `opencode` | Heuristic | Held and tagged, so it can report once opencode grows something hook-shaped. Wears atrium's [[Themes\|theme]] |
| `codex` | Heuristic | Same |
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

## Claude's hooks

**Registered through `--settings`, never `~/.claude/settings.json`.** Each agent
is launched with its hooks passed on the command line, so a Claude started
outside atrium is completely unaffected and the global config is never written.

Nine events are registered, one hook each: `SessionStart`, `UserPromptSubmit`,
`Notification`, `PermissionRequest`, `PostToolUse`, `PermissionDenied`, `Stop`,
`StopFailure`, `SessionEnd`. See [[Status]] for what each one means.

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
hooks.json    one hook per event
```

It is rewritten on every spawn, because the path it carries is wherever this
atrium is installed.

### copilot's event names are translated, not adopted

atrium's wire vocabulary is spelled the way claude spells it, and
`Status::from_hook_event` is the one table that reads it. So the adapter
translates on the way out rather than teaching `core` a second vocabulary:

| copilot says | atrium hears | which means |
| --- | --- | --- |
| `sessionStart` | `SessionStart` | idle |
| `userPromptSubmitted` | `UserPromptSubmit` | working |
| `notification` | `Notification` | **needs you** |
| `postToolUse`, `postToolUseFailure` | `PostToolUse` | lifts a wait |
| `agentStop` | `Stop` | idle |
| `errorOccurred` | `StopFailure` | error |
| `sessionEnd` | `SessionEnd` | exited |

`notification` is the one that earns the wiring: it is what copilot raises for a
permission prompt or a question of its own, which is the whole thing the sidebar
exists to show.

### A shell string is quoted, because it cannot be avoided

copilot's hook command is a **shell string**, under a `bash` key. Claude's
`args` escape hatch is not available, so the path is single-quoted and any
quote in it is closed, escaped and reopened — `helpers::shell::quote`. A test
asserts it with a path holding a space, a quote and a `rm -rf` after it.

Only `bash` is written. The status channel is a unix socket, so there is no
Windows to write a `powershell` arm for.

`timeoutSec` is 5. copilot's own default is thirty seconds and there is no
`async` flag to lean on, but the handler writes one line to a socket and exits.

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
[[Status|"needs you"]] that pulsed for as long as the atrium was up. Knocking
also settles the question the pid never could, since a pid the system has handed
out again reads as alive.

The uid behind the fallback directory is the owner of the home directory, taken
with `MetadataExt::uid`. Not libc, so it still costs no dependency, and not
`/proc/self/status`, which answered `0` for everyone off Linux.

One short-lived thread serves each connection, so a client that connects and
then stalls cannot hold up the ones behind it. The draw loop drains what has
arrived without ever blocking.

## The wire format

One line, tab separated:

```
<agent id>\t<event>\n
```

There is nothing else to carry, because the event name arrived as an argument.
Malformed lines are rejected rather than guessed at.

Reports name an agent **by id**, so they land correctly even after rows have
been reordered or closed — see [[Registry and focus]].

## Unix only, deliberately

The status channel is a unix socket. Windows would need a different transport,
and nothing here needs it yet.
