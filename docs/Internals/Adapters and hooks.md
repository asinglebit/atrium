# Adapters and hooks

How an agent tells atrium what it is doing. `src/adapters/`, `src/ipc/`.

## Picking an adapter

`detect(program)` matches on the program name with any directories stripped off,
so `/usr/local/bin/claude` and `claude` reach the same adapter.

| Name | Reports via | Notes |
| --- | --- | --- |
| `claude` | Hooks | Fully wired. Wears atrium's [[Themes\|theme]], and follows a change while running |
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

Seven events are registered, one hook each: `SessionStart`, `UserPromptSubmit`,
`Notification`, `PermissionRequest`, `Stop`, `StopFailure`, `SessionEnd`. See
[[Status]] for what each one means.

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

## The socket

Lives in `$XDG_RUNTIME_DIR/atrium/<pid>-<n>.sock`, falling back to a temp
directory keyed by uid. `XDG_RUNTIME_DIR` is a tmpfs, so nothing survives a
reboot to be mistaken for a live atrium. The counter distinguishes servers
within one process, which the pid alone cannot do.

**Stale sockets are swept on the way in.** An atrium that is `kill`ed rather
than quit never runs its `Drop`, so its socket outlives it. Binding clears any
whose pid is no longer alive, which keeps the directory from filling up. That
check reads `/proc`, so it is a no-op off Linux — as is reading the uid, which
is done from `/proc/self/status` rather than libc so the fallback path costs no
dependency.

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
