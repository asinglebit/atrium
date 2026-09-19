# Configuration

Three files, in `~/.config/atrium/`. **One of them is yours, two are atrium's.**

| File | Written by | Holds |
| --- | --- | --- |
| `config.toml` | You | Theme name and key bindings |
| `theme.json` | atrium | The actual colours — see [[Themes]] |
| `layout.json` | atrium | The [[Sidebar]] width |

atrium never writes `config.toml`. A config that rewrites itself is a config you
cannot keep in a dotfiles repo.

## config.toml

All of it optional:

```toml
[theme]
name = "one dark warmer"   # any of the 47 preset names

[keys]
quit = "ctrl+q"
goto = "ctrl+g"
settings = "ctrl+s"
sidebar = "ctrl+o"
new = "ctrl+t"
dismiss = "ctrl+x"
next = "ctrl+n"
previous = "ctrl+p"
```

See [[Keys]] for how a chord is written, and for the constraint that a binding
has to be one a terminal can actually deliver.

Anything the file gets wrong is **kept rather than discarded**, so it can be
reported instead of failing silently. The TUI carries on with the defaults.

## layout.json

```json
{
  "sidebar_width": 52
}
```

Written when a drag settles — see [[Mouse]]. A file that is missing, unreadable
or malformed is not a problem: the defaults are a perfectly good answer, and a
bad one is overwritten by the next drag. Nothing here is worth failing over —
losing a dragged width costs one drag, taking atrium down over it costs an agent.

## Checking it

```sh
atrium --check-config
```

Says what atrium actually loaded — the claimed chords, the theme, the projects
root — and then names anything it could not use, exiting non-zero if there was
anything. A config that does nothing can be diagnosed without starting the TUI
over it.

## Environment

| Variable | Effect |
| --- | --- |
| `ATRIUM_PROJECTS` | Where [[Projects]] are looked for. Defaults to `~/projects` |
| `XDG_RUNTIME_DIR` | Where the status socket lives — see [[Adapters and hooks]] |
| `ATRIUM_AGENT_ID`, `ATRIUM_SOCK` | Set *by* atrium on each agent, not by you |
