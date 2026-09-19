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

## Statuses use the palette

[[Status]] colours are palette entries — `COLOR_RED`, `COLOR_AMBER`,
`COLOR_GREEN`, `COLOR_GREY_400`, `COLOR_GREY_600` — rather than colours of
atrium's own, so the two tools can never disagree about what red is.

atrium's own accent is `COLOR_PURPLE` — the `atrium` on the title line, and the
lower part of the wordmark in [[Settings]] and on the [[Splash]], whose top 30%
takes `COLOR_PINK` above it.
