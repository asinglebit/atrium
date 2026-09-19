# atrium

A terminal UI that **holds coding agents**. Start it in any terminal; it spawns
agent CLIs as its own children and keeps them in a sidebar, showing what each
one is doing. Switch between them inside atrium.

It is not a multiplexer and it is not tmux-aware. Run it inside tmux, inside
ssh, on a bare TTY — atrium neither knows nor cares. Run a second one in another
pane and it holds its own, independent set.

```
  atrium |  ~/projects/personal/atrium                          agents
╭────────────────────────────────────────────────────────────────────────╮
│ agents 3          │ ▐▛███▛█   Claude Code v2.1.278                     │
│ ⠙ 1 atrium master*│▝▜██████▀  Opus 5 (1M context)                      │
│ ● 2 guitar   main │  ▝▝ ▝▝    ~/projects/personal/guitar               │
│ ○ 3 bazzite  sway │                                                    │
│                   │ ❯ reply with exactly: pong                         │
│                   │ ● pong                                             │
╰────────────────────────────────────────────────────────────────────────╯
  guitar working  ● main                                              2/3
```

Title line, rounded frame, bordered panes, status line — guitar's chrome, off
the same `theme.json`. Retheme one and the other follows.

## Install

```sh
git clone git@github.com:asinglebit/atrium.git ~/projects/personal/atrium
cd ~/projects/personal/atrium
cargo build --release
```

The binary is `target/release/atrium`. It needs to stay somewhere stable,
because agents call back into it by absolute path — see below.

## Use

```sh
atrium                 # hold a claude in the current directory
atrium bash --norc     # hold anything else
atrium --check-config  # what the config says, and what it got wrong
```

Everything you type goes to the focused agent, except chords behind the leader,
`F12`:

| | |
| --- | --- |
| `F12 n` | hold a new agent — pick a project, pick a CLI |
| `F12 x` | dismiss this one; dismissing the last ends atrium |
| `F12 j` / `F12 k` | next / previous |
| `F12 1`…`F12 9` | jump straight to that agent |
| `F12 q` | quit |
| `F12 F12` | send the leader itself to the agent |

## Config

`~/.config/atrium/config.toml`, all of it optional:

```toml
[theme]
name = "one dark warmer"   # any of guitar's ~30 preset names

[keys]
quit = "ctrl+q"            # any key or chord: "esc", "ctrl+y", "alt+enter"
goto = "ctrl+g"
new = "ctrl+t"
dismiss = "ctrl+]"
next = "ctrl+n"
previous = "ctrl+p"
```

Individual colours are **not** set here: they live in `theme.json`, shared with
guitar, so the two stay in step. With no config at all atrium uses whatever
theme guitar is set to.

`--check-config` names anything it could not use rather than failing silently.
Projects come from `$ATRIUM_PROJECTS`, else `~/projects`.

## Layout

| Path | What it does |
| --- | --- |
| `src/core/pty.rs` | Spawns an agent on a pty, reads it into a `vt100::Parser`, resizes both halves |
| `src/core/agent.rs` | One held agent: identity, status, git context |
| `src/core/registry.rs` | The held set, the focus, and routing status reports by id |
| `src/core/projects.rs` | Finds git repositories under the projects root |
| `src/core/git.rs` | Branch and dirty flag for a sidebar row |
| `src/core/config.rs` | Reads `config.toml`, keeping a list of what it got wrong |
| `src/adapters/` | Per-CLI launch and status wiring — `claude`, and stubs for `opencode` and `codex` |
| `src/ipc/server.rs` | The unix socket agents report back through |
| `src/ipc/hook.rs` | The other end: `atrium hook <Event>` |
| `src/app/draw/` | The sidebar, the stage, and the new-agent modal |
| `src/app/input/keys.rs` | Turns a crossterm key into the bytes a terminal would have sent |
| `src/tests/` | Mirrors the tree above; attached with `#[path]` from each source file |

Built on `ratatui`, `tui-term`/`vt100` for the embedded terminal, `portable-pty`
for the children, and `git2` for the row context.

## Things that are load-bearing, and why

**Agents die with atrium, and that is the design.** atrium owns the pty, so when
it exits the children go with it. Surviving the terminal is exactly what would
require a background server, which is the thing this avoids: tmux already does
persistence, and duplicating it is how you end up writing a multiplexer. If it
ever matters, the change is to split a server out behind the same sidebar.

**Actions fire directly, so every binding is a key taken from the agent.**
There is no leader to press first, which makes the choice of defaults the whole
design: `ctrl+letter` is a crowded space, and anything atrium claims the agent
never sees. The five defaults are picked for what they cost rather than for the
mnemonic — `ctrl+]` and `ctrl+q` cost essentially nothing, `ctrl+n`/`ctrl+p`
cost shell history that Claude's own input box does not use, and `ctrl+t` costs
readline's transpose. A test asserts no default lands on `ctrl+c`, `ctrl+d`,
`ctrl+z`, `ctrl+v`, `ctrl+x`, `ctrl+l`, `ctrl+r`, `ctrl+u`, `ctrl+w`, `ctrl+a`,
`ctrl+e`, `ctrl+k` or `ctrl+[`.

Nothing above atrium contends for these: sway is `Super+…` only, ghostty is
`ctrl+shift+…`, and tmux claims `C-a` plus thirteen prefix-less `M-` bindings —
no plain `ctrl+letter` among them. The one to avoid is `ctrl+a`, which tmux
takes as its prefix and atrium would therefore never receive.

**Hooks go in through `--settings`, never `~/.claude/settings.json`.** Each
agent is launched with its hooks passed on the command line, so a Claude started
outside atrium is completely unaffected and the global config is never written.
The agent also carries `ATRIUM_AGENT_ID` and `ATRIUM_SOCK` in its environment,
which is how `atrium hook` knows who it is and where to report.

**The hook handler is registered in exec form, and that is a correctness fix,
not a preference.** Claude's `command` field is a shell string. Written that
way, a binary path containing a quote escapes its own quoting —
`"/tmp/ev"il" hook Stop` — even though the surrounding JSON is perfectly valid.
Passing `args` instead runs the handler directly with no shell, so no path can
be re-split or broken out of. A test asserts this with a path containing both a
space and a quote.

**The event name is an argument, so the handler never parses JSON.** Hooks are
registered one per event, which means `atrium hook Stop` already knows what
happened and can ignore the payload on stdin entirely. That is the whole reason
this crate has no JSON parser. It still drains stdin, to avoid an EPIPE in the
agent that wrote it.

**A hook that fails is a hook that interrupts the agent.** `atrium hook` never
reports an error: no socket, no env, a dead atrium — it exits 0 and says
nothing. A missed status update is not worth disturbing a conversation over.

**Every agent is resized, not just the focused one.** A background agent whose
pty still thinks it is the old size renders wrong for a frame when you switch to
it. Resizing all of them costs nothing and removes the flicker.

**Dropping the slave pty is what makes EOF happen.** Hold onto it and the reader
thread blocks forever after the child exits, and the agent never shows as gone.

**Key releases are filtered out.** Terminals speaking the kitty keyboard
protocol send press *and* release; without the filter every keystroke reaches
the agent twice.

**Sockets from a killed atrium are swept on the way in.** An atrium that is
`kill`ed rather than quit never runs its `Drop`, so its socket outlives it.
Binding clears any whose pid is no longer alive, which keeps the directory from
filling up. (This reads `/proc`, so it is a no-op off Linux.)

**A CLI that is not installed is a message, not the end of atrium.** Picking
`codex` when there is no codex used to return an error that propagated out of
the run loop and took the whole UI down. The failure now stays inside the modal
so another choice can be made.

**The picker ranks, it does not just filter.** Loose matching means `gui` finds
both `guitar` and `asinglebit.github.io` — the letters really are in there, in
order. Matches are scored by how far apart the matched characters sit, so the
tight one wins. Without that the obvious answer is rarely first.

**Git context is read on a timer, never in the draw loop.** `git status` on a
large repository is far too slow to run at frame rate. Three seconds is slow
enough to be free and fast enough that the `*` appears while you still care.

**Unix only, deliberately.** The status channel is a unix socket. Windows would
need a different transport, and nothing here needs it yet.

**The palette is guitar's, copied verbatim, and the theme file is shared.**
`src/helpers/palette.rs` is guitar's file with one function changed:
`theme_path()` reads atrium's own `theme.json` if there is one and **guitar's
otherwise**. So retheme guitar and atrium follows, with nothing to configure —
which is the point. Statuses map onto that palette (`COLOR_RED`, `COLOR_AMBER`,
`COLOR_GREEN`) rather than carrying colours of their own, so the two tools can
never disagree about what red is.

**The chrome is guitar's, in the same order guitar draws it:** background, then
a rounded frame around everything, then a title line above it and a status line
below, then the panes inside. A pane draws only the one edge that separates it
from the next — the frame already supplies the rest — so the line between the
sidebar and the stage is single rather than doubled. Rows are zebra-striped with
`zebra_list_items`, lifted from guitar's `pane_window.rs`: selected row in
`COLOR_GREY_800`, every other row in `COLOR_GREY_900`.

**`ctrl+1`..`ctrl+9` is not a thing, which is why there is a goto list.** A
terminal has no legacy encoding for ctrl and a digit: `ctrl+1` arrives as a bare
`1`, indistinguishable from typing it, and `ctrl+2` arrives as NUL. tmux ships
`extended-keys off` and does not model the key for `send-keys` either. So the
numbers on the rows are jumped to from inside `ctrl+g`, where the modal owns the
keyboard and a plain digit means what it says.

**Test files are attached with `#[path]`, which is a footgun with a guard.**
Rewriting a source file without its `#[cfg(test)] mod tests;` block removes its
whole test file from the build, and the suite still passes — so nothing tells
you. That happened here to the sidebar. `src/tests/tree.rs` now walks the tree
and fails if any test file is not attached to something.
