# Code map

```
src/
  main.rs              CLI: help, version, --check-config, `hook`, then the TUI
  lib.rs               The module tree

  core/
    agent.rs           One held agent: identity, status, git context
    pty.rs             The pty, the parsed screen, the child, and killing it
    registry.rs        The held set, the focus, routing reports by id
    git.rs             Branch and dirty flag for a row
    projects.rs        Finding git repositories under the projects root
    config.rs          Reads config.toml, keeping a list of what it got wrong
    layout_config.rs   The sidebar width, in layout.json

  adapters/            Per-CLI launch and status wiring
    claude.rs          --settings hooks, in exec form
    opencode.rs        Tagged only
    codex.rs           Tagged only

  ipc/
    server.rs          The unix socket agents report back through
    hook.rs            The other end: `atrium hook <Event>`
    wire.rs            One tab-separated line

  app/
    app.rs             The run loop, input dispatch, and what owns the keyboard
    draw/              sidebar, stage, settings, menu, pane, title, statusbar
    draw/modals/       new_agent, goto
    input/keymap.rs    Chords, parsing, and the eight actions
    input/keys.rs      A crossterm key back into terminal bytes
    state/             layout, menu, settings, picker, goto

  helpers/
    palette.rs         guitar's palette, with two functions changed
    logo.rs            The wordmark
    scroll.rs          Keeping a selection on screen, and scrollbar lengths
    text.rs            Truncation from either end
    spinner.rs         Six frames, from elapsed time
    json.rs            String escaping, for writing the hook settings
    version.rs

  tests/               Mirrors the tree above
```

## The test convention

Test files live in `src/tests/`, mirroring the source tree, and are attached
from each source file:

```rust
#[cfg(test)]
#[path = "../../tests/app/state/menu.rs"]
mod tests;
```

**This is a footgun with a guard.** Rewriting a source file without its
`#[cfg(test)] mod tests;` block removes its whole test file from the build, and
the suite still passes — so nothing tells you. That happened here to the
sidebar. `src/tests/tree.rs` now walks the tree and fails if any test file is
not attached to something.

## CI

`cargo fmt --check`, `cargo clippy --all-targets -- -D warnings`, `cargo test`,
then a release build on Linux and macOS. Tagging `v*.*.*` cuts a release with
both binaries attached. Unix only — see [[Adapters and hooks]].

## Conventions taken from guitar

Same author, same shape: `rustfmt.toml`, edition 2024, the `src/tests/` mirror,
`#[allow(clippy::module_inception)]` on `src/app/mod.rs`, and the release
profile (fat LTO, one codegen unit, stripped debuginfo).
