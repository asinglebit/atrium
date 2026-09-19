# Themes

47 presets, shared with guitar. `src/helpers/palette.rs` is guitar's file with
two functions changed.

## Reading, and the fallback to guitar

`theme_path()` reads **atrium's own `theme.json` if there is one, and guitar's
otherwise**. So retheme guitar and atrium follows, with nothing to configure —
which is the whole reason the two look like one tool.

## Writing, and why it is not the same path

`save_theme()` **always** writes atrium's own path. Picking a theme in atrium's
[[Settings]] must not retheme guitar behind its back, which is exactly what the
unmodified function would have done.

So the fallback is one-way: atrium will follow guitar, but never edit it.

## Picking one

Themes are easier picked than typed. `ctrl+s` opens [[Settings]], where the
themes tab lists all 47 and `enter` applies one. `config.toml` takes a preset
**name**; individual colours are not set there, they live in `theme.json`.

## The theme reaches the agents too

An agent paints its own cells, so atrium's palette would normally stop at the
frame. Both CLIs atrium knows how to theme are told instead, and every agent it
holds is launched wearing the selected theme.

| | Where the theme goes | Picked by | A change while running |
| --- | --- | --- | --- |
| claude | `<CLAUDE_CONFIG_DIR>/themes/atrium.json` | `"theme": "custom:atrium"` in the `--settings` document | **Followed** — claude watches the file and repaints |
| opencode | `~/.config/opencode/themes/atrium.json` | `~/.config/atrium/opencode-tui.json`, handed over as `OPENCODE_TUI_CONFIG` | Not followed; the next opencode gets it |

## Claude

The theme is **overrides on a base**: `{ "name", "base", "overrides" }`, where
`base` is `dark` or `light` — chosen by the brightness of the theme's own
background — and the overrides are the couple of dozen colours atrium has an
opinion about. Everything else is left to the base, so a colour claude adds
later is not a hole atrium has to fill.

Values go over as `rgb(r,g,b)`, or `ansi:name` for one of the terminal's own.
Anything that cannot be said that way — `Reset`, an indexed colour — is left out
rather than guessed at, and the base keeps its own answer for that key.

It is written **per subscription**, because `CLAUDE_CONFIG_DIR` is the directory
claude reads themes from: holding `work` writes `~/.claude-work/themes/`, and a
profile naming no directory writes claude's own default, `~/.claude/themes/`.
Claude's own mark stays in the theme's orange rather than taking atrium's
purple: it is whose agent it is, not whose window.

## Opencode

opencode takes two files, because it reads a theme **by name from its own config
directory and nowhere else** — not from a path in the config, and not from
`OPENCODE_CONFIG_DIR`, both of which were tried:

| File | Whose | What it is |
| --- | --- | --- |
| `~/.config/opencode/themes/atrium.json` | atrium writes it | The palette, as an opencode theme |
| `~/.config/atrium/opencode-tui.json` | atrium owns it | `{ "theme": "atrium" }`, handed over as `OPENCODE_TUI_CONFIG` |

The second is why your own `tui.json` is never edited: `OPENCODE_TUI_CONFIG` is
read **in addition** to it, so the keybinds and scroll settings in it still
apply. The first is the only file atrium writes outside its own directory, it is
named after atrium, and the [[Settings]] general tab says where it is whenever
opencode is installed.

Colours go over as opencode takes them: `#rrggbb` for an RGB one, a bare number
for one of the terminal's own sixteen, and `none` — drawn transparent — for
anything the theme leaves to the terminal.

Handing the **index** over is the honest answer for the `ansi` preset: a hex
guess would be atrium deciding what your terminal's red is. opencode then draws
its own idea of that index rather than asking the terminal, so `ansi` is the one
preset that can look a little different inside an agent than around it.

**A running opencode keeps the theme it started with.** It reads the file once
and watches neither it nor the tui config — both were tried — and its TUI takes
no theme over its own API, which offers a picker and a dark/light toggle and
nothing else. So a switch reaches the opencodes held after it. The file is still
rewritten at once, so the next one is already right.

## Statuses use the palette

[[Status]] colours are palette entries — `COLOR_RED`, `COLOR_AMBER`,
`COLOR_GREEN`, `COLOR_GREY_400`, `COLOR_GREY_600` — rather than colours of
atrium's own, so the two tools can never disagree about what red is.

atrium's own accent is `COLOR_PURPLE` — the `atrium` on the title line, and the
lower part of the wordmark in [[Settings]] and on the [[Splash]], whose top 30%
takes `COLOR_PINK` above it.
