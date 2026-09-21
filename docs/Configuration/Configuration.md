# Configuration

Four files, in `~/.config/atrium/`. **One of them is yours, three are
atrium's.**

| File | Written by | Holds |
| --- | --- | --- |
| `config.toml` | You | Theme name and key bindings |
| `theme.json` | atrium | The actual colours — see [[Themes]] |
| `layout.json` | atrium | The [[Sidebar]] width |
| `profiles.json` | atrium | The [[Profiles]], and which is default |

atrium never writes `config.toml`. A config that rewrites itself is a config you
cannot keep in a dotfiles repo — which is exactly why profiles are not in it any
more: they are edited in [[Settings]], and a TOML rewrite would drop your
comments and reformat everything around them.

## config.toml

All of it optional:

```toml
[theme]
name = "one dark warmer"   # any of the 60 preset names

[keys]
action = "ctrl+space"
quit = "q"
goto = "space"
settings = "?"
sidebar = "shift+1"   # or "!", which is the same keystroke
new = "n"
dismiss = "x"
next = "j"
previous = "k"
```

There is no entry for the digits: `1`…`9` and `0` name the agent on that row
and are not rebindable. See [[Keys]] for how a chord is written, and for the
constraint that a binding has to be one a terminal can actually deliver.

A file that still carries a `[[profiles]]` block or a `default` is **told where
they went** rather than having them silently ignored — `--check-config` names
both.

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

## profiles.json

```json
{ "default": "work",
  "profiles": [{ "name": "work", "config_dir": "~/.claude-work", "args": [] },
               { "name": "day-job", "program": "copilot", "config_dir": "~/.copilot-day-job" }] }
```

Written whenever a profile is added, renamed or deleted in [[Settings]]. Unlike
the theme and the layout, a failure here is **reported**: it is written in
answer to something you just did, so silence would look like the edit had
worked. See [[Profiles]].

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
| `CLAUDE_CONFIG_DIR` | Set by atrium from a claude profile's `config_dir`, which is what picks a subscription |
| `COPILOT_HOME` | The same, for a copilot profile. Which variable a `config_dir` sets follows the CLI |
