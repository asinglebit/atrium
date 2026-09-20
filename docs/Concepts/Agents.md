# Agents

An agent is one CLI running on its own pty, owned by atrium. `src/core/agent.rs`
holds the identity; `src/core/pty.rs` holds the terminal underneath it.

## What one is made of

| Field | Where it comes from |
| --- | --- |
| `id` | A process-wide counter starting at 1. Unique within one atrium, which is all a [[Adapters and hooks\|hook report]] needs |
| `name` | The working directory's own last component, falling back to the program name when the path has none |
| `cwd` | Where it was started, which is also what the title line shows |
| `status` | See [[Status]] |
| `git` | Branch and dirty flag, re-read on a timer — see [[Registry and focus]] |
| `session` | The pty, the parsed screen, and the child |

An `AgentSpec` (program, args, cwd) is kept rather than a `CommandBuilder`,
because `CommandBuilder` cannot be cloned and "another one of these" has to be
possible.

## The pty underneath

`PtySession::spawn` opens a pty pair, spawns the child on the slave, and then
**drops the slave**. That drop is load-bearing: hold onto it and the reader
thread blocks forever after the child exits, so the agent never shows as gone.

The child is told it is talking to an xterm, because that is what the parser
understands:

```
TERM=xterm-256color
COLORTERM=truecolor
```

A background thread reads the master into a `vt100::Parser` shared with the draw
loop behind a mutex. The parser keeps **10,000 lines** of scrollback per agent.

Resizing resizes both halves — the kernel, which signals the child, and the
parser, which decides how the screen reflows.

## Ending one

`PtySession`'s `Drop` kills the child and then waits on it. Both halves matter:

- Closing the pty on its own only **hangs the child up**, and a CLI is free to
  ignore that. Without the kill, a closed agent could keep running.
- Without the wait, a killed agent would sit as a zombie for as long as atrium
  runs.

So dropping a session is what ends an agent, which is what makes `ctrl+space` `x` (see
[[Keys]]) and the menu's *close* entry actually close something.

## Agents die with atrium

atrium owns the pty, so when it exits the children go with it. This is
deliberate rather than a limitation: surviving the terminal is exactly what
would require a background server, and tmux already does persistence. If it ever
matters, the change is to split a server out behind the same sidebar.
