# atrium

> **Very much a work in progress.** It is being built as it is used, so the
> shape still moves week to week. Bindings, config files and layout are all
> fair game to change, and nothing here is stable yet.

A terminal UI that **holds coding agents**. Start it in any terminal; it spawns
agent CLIs as its own children and keeps them in a sidebar, showing what each
one is doing. Switch between them inside atrium.

Same family as [guitar](https://github.com/asinglebit/guitar) — the same chrome,
the same palette, the same `theme.json` — pointed at a different job. guitar is
for reading a repository; atrium is for running the agents that change one. Both
are written for one person's day-to-day work first, and that is the whole design
brief here: a day spent driving several coding agents at once, wanting to see at
a glance which one is working, which one is waiting on you, and what repository
each is standing in. The defaults are opinionated because they are one person's
defaults, and the scope stops where that day's work stops.

It is not a multiplexer and it is not tmux-aware. Run it inside tmux, inside
ssh, on a bare TTY — atrium neither knows nor cares. Run a second one in another
pane and it holds its own, independent set.

```
  atrium |  ~/projects/personal/atrium                                            agents
╭────────────────────────────────────────────────────────────────────────────────────────╮
│ ⠙ 1 atrium                         master* │ ▐▛███▛█   Claude Code v2.1.278            │
│ ● 2 guitar                            main │▝▜██████▀  Opus 5 (1M context)             │
│ ○ 3 bazzite                           sway │  ▝▝ ▝▝    ~/projects/personal/guitar      │
│                                            │                                           │
│                                            │ ❯ reply with exactly: pong                │
│                                            │ ● pong                                    │
╰────────────────────────────────────────────────────────────────────────────────────────╯
  guitar working  ● main                                                               2/3
```

Title line, rounded frame, bordered panes, status line — guitar's chrome, off
the same `theme.json`. Retheme one and the other follows.

`docs/` is an Obsidian vault covering all of this in more detail. Open the
folder as a vault, or start at [docs/Atrium.md](docs/Atrium.md) and read it as
plain markdown.

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

Everything you type goes to the focused agent, except these, which fire
directly — there is no leader to press first:

| | |
| --- | --- |
| `ctrl+t` | hold a new agent — pick a project, pick a CLI |
| `ctrl+x` | close this one; closing the last ends atrium |
| `ctrl+n` / `ctrl+p` | next / previous |
| `ctrl+g` | go to, where `1`…`9` jump straight to a row |
| `ctrl+o` | show or hide the sidebar |
| `ctrl+s` | settings |
| `ctrl+q` | quit |

The mouse works too. Click a row to put it on the stage, wheel to scroll, and
drag the line between the panes to resize the sidebar — the width is remembered.
**Right-click anywhere for a menu**: on a row it offers that agent, anywhere
else it offers the rest. Inside the agent every other mouse event is forwarded
on, so the agent's own mouse support keeps working.

## Config

`~/.config/atrium/config.toml`, all of it optional:

```toml
[theme]
name = "one dark warmer"   # any of guitar's ~30 preset names

[keys]
quit = "ctrl+q"            # any key or chord: "esc", "ctrl+y", "alt+enter"
goto = "ctrl+g"
settings = "ctrl+s"
sidebar = "ctrl+o"
new = "ctrl+t"
dismiss = "ctrl+x"
next = "ctrl+n"
previous = "ctrl+p"
```

Themes are easier picked than typed: `ctrl+s` opens settings, where the themes
tab lists all thirty and Enter applies one. Individual colours are **not** set
in `config.toml`: they live in `theme.json`.

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
| `src/core/layout_config.rs` | The one thing atrium writes back: the sidebar's width, in `layout.json` |
| `src/adapters/` | Per-CLI launch and status wiring — `claude`, and stubs for `opencode` and `codex` |
| `src/ipc/server.rs` | The unix socket agents report back through |
| `src/ipc/hook.rs` | The other end: `atrium hook <Event>` |
| `src/app/draw/` | The sidebar, the stage, the settings view, the menu and the modals |
| `src/app/state/layout.rs` | Where the sidebar, stage, title line and status line go |
| `src/app/state/menu.rs` | What the right-click menu offers, and where its box lands |
| `src/helpers/logo.rs` | The wordmark the settings view is headed with |
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
never sees. The defaults are picked for what they cost rather than for the
mnemonic — `ctrl+q` and `ctrl+s` are XON and XOFF and raw mode has already
turned flow control off, `ctrl+n`/`ctrl+p` cost shell history that Claude's own
input box does not use, `ctrl+t` costs readline's transpose, and `ctrl+x` costs
a two-key readline prefix whose second key an agent's input box does not
implement. A test asserts no default lands on `ctrl+c`, `ctrl+d`, `ctrl+z`,
`ctrl+v`, `ctrl+l`, `ctrl+r`, `ctrl+u`, `ctrl+w`, `ctrl+a`, `ctrl+e`, `ctrl+k`
or `ctrl+[`.

**A default also has to be a chord a terminal can actually deliver, which is
narrower than it looks.** `ctrl+]` was the default for closing an agent and
never once fired. atrium does not push the kitty keyboard flags, so it reads the
legacy encoding, and there crossterm maps the bytes `0x1C`..`0x1F` onto
`ctrl+4`..`ctrl+7` — `ctrl+]` is `0x1D`, so it arrives as `ctrl+5` and a chord
written `ctrl+]` can never match. Outside a letter a terminal has a byte for
almost nothing, which is the same fact behind `ctrl+1`..`ctrl+9` below. A test
now asserts every default is a ctrl **letter**.

**Closing an agent kills it, because letting go of the pty does not.** Dropping
a session closes atrium's end, which only hangs the child up — a CLI is free to
ignore that. `PtySession`'s `Drop` kills the child and then waits on it, so one
closed agent is one ended process and no zombie left behind for as long as
atrium runs. A test spawns a `sleep`, drops its session, and watches the pid
leave `/proc`.

**The sidebar squeezes before it disappears.** It opens at guitar's
`LAYOUT_WIDTH_LEFT_PANE`, 45 columns, so the two tools open at the same
proportions. That is wide enough that a fixed 45 would have left an 80-column
terminal with no sidebar at all, so the width is clamped the way guitar clamps
its own: down to 16 columns before it is dropped, and never far enough to take
the agent below 40. Dragging the line between the panes sets it, and
`layout.json` remembers it — written when the drag settles rather than on every
frame of it, so one resize is one write. `config.toml` stays a file only you
write.

**The right-click menu claims a button that was doing nothing.** Mouse capture
already suppresses the terminal's own menu, and the agent ignores a forwarded
right-click, so before this there was no way to reach an action with the mouse —
and none at all once the sidebar was hidden. On a row the menu offers that
agent by name; anywhere else it offers what can be done regardless. It quotes
the chord beside each entry, so it doubles as the place the keys are learnt.

**The logo is purple, and it stands where guitar's heatmap stands.** guitar
heads its settings with a contribution graph and lines every row up to its
width; atrium has no commits to plot, so the wordmark takes that place and the
column lines up to it. The two purples split across the rows the way guitar
splits its own logo across two greens, and the `atrium` in the title line is the
brighter of them.

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

**The palette is guitar's, copied verbatim, and the theme file is read from
guitar but never written to it.** `src/helpers/palette.rs` is guitar's file with
two functions changed. `theme_path()` reads atrium's own `theme.json` if there
is one and **guitar's otherwise**, so retheme guitar and atrium follows with
nothing to configure. `save_theme()` always writes atrium's own path — picking a
theme in atrium's settings must not retheme guitar behind its back, which is
exactly what the unmodified function would have done. Statuses map onto that palette (`COLOR_RED`, `COLOR_AMBER`,
`COLOR_GREEN`) rather than carrying colours of their own, so the two tools can
never disagree about what red is.

**A pane carries no title, because the title costs the top row.** The sidebar
used to be headed `agents 3`; the status line already says `2/3`, so the row was
paying for something said twice. Losing it moves the first agent up to the
pane's own top row, which `row_at` and the scroll trap are measured against.

**The chrome is guitar's, in the same order guitar draws it:** background, then
a rounded frame around everything, then a title line above it and a status line
below, then the panes inside. A pane draws only the one edge that separates it
from the next — the frame already supplies the rest — so the line between the
sidebar and the stage is single rather than doubled. Rows are zebra-striped with
`zebra_list_items`, lifted from guitar's `pane_window.rs`: selected row in
`COLOR_GREY_800`, every other row in `COLOR_GREY_900`.

**The line between the panes is the scrollbar.** guitar's scrollbar uses the
border glyph as its track and `▌` as the thumb, so one column is both the
separator and the scroll position. ratatui draws *nothing* for a scrollbar whose
content length is zero, though, so a list that fits would lose the separator
entirely — `draw_gutter` falls back to a plain bordered block in that case.

**`ctrl+1`..`ctrl+9` is not a thing either, which is why there is a goto list.** A
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
