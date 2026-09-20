# Keys

**Everything you type reaches the focused agent.** atrium's own actions sit
behind one prefix, `ctrl+a`, which is guitar's action mode.

| Gesture | Action |
| --- | --- |
| `ctrl+a` `n` | hold a new agent |
| `ctrl+a` `x` | close this one; closing the last goes back to the [[Splash]] |
| `ctrl+a` `j` / `k` | next / previous |
| `ctrl+a` `g` | go to, where `1`…`9` jump straight to a row |
| `ctrl+a` `1` | show or hide the [[Sidebar]] |
| `ctrl+a` `?` | [[Settings]] |
| `ctrl+a` `q` | quit |

The prefix and all eight keys are rebindable — see [[Configuration]], where the
prefix is the `action` entry.

## Press it twice inside tmux

`ctrl+a` is tmux's prefix here, so the first one never leaves tmux. The second
does, because the config carries `bind C-a send-prefix`. That is already how
guitar's action mode is driven on this machine, so the gesture is not a new one
to learn.

## Why a prefix, and why this one

atrium used to fire eight `ctrl+letter` chords directly. That meant eight keys
an agent could never see, and the choice of defaults was the whole design:
`ctrl+t` cost readline's transpose, `ctrl+n`/`ctrl+p` cost shell history,
`ctrl+x` cost a two-key prefix, and so on. **They are all the agent's again.**

What is left is one key. `ctrl+a` costs readline's start-of-line — and inside
tmux it costs nothing at all, because tmux was already taking it and the agent
never saw it. The keys behind the prefix are bare letters and cost nothing
either: they mean something only in the moment after it.

The letters are guitar's wherever guitar has one, so the two tools do not
disagree: `1` toggles a pane, `?` is settings, `x` drops a thing, `q` exits,
`j`/`k` walk a list.

A test asserts exactly one chord is claimed, that every action key is
unmodified, and that the claimed one is none of `ctrl+c`, `ctrl+d`, `ctrl+z`,
`ctrl+v`, `ctrl+l`, `ctrl+r`, `ctrl+u`, `ctrl+w`, `ctrl+e`, `ctrl+k` or
`ctrl+[`, which is Escape.

## A key that means nothing cancels

Press the prefix and then something unbound and nothing happens — the key is
swallowed rather than passed on. Half a mistyped gesture landing in a
conversation is worse than nothing happening. The [[Stage|status line]] shows
`ctrl+a` while atrium waits for the second key, since that is the only moment a
keystroke means something other than itself.

Nothing above atrium contends for the prefix: sway is `Super+…`, ghostty is
`ctrl+shift+…`, and tmux's own `C-a` is the one being deliberately shared.

## A binding also has to be deliverable

This is narrower than it looks, and it is the trap this project has fallen into
twice.

atrium does not push the kitty keyboard flags, so it reads the **legacy
encoding**. There, a terminal has a byte for `ctrl` plus a letter and for very
little else:

| What you press | What arrives |
| --- | --- |
| `ctrl+a`…`ctrl+z` | `0x01`…`0x1A` → the letter, with CONTROL |
| `ctrl+]` | `0x1D` → **`ctrl+5`** |
| `ctrl+\`, `ctrl+^`, `ctrl+_` | `0x1C`, `0x1E`, `0x1F` → `ctrl+4`, `ctrl+6`, `ctrl+7` |
| `ctrl+space` | `0x00` → `ctrl+space` |
| `ctrl+1` | a bare `1` |
| `ctrl+2` | NUL |

**`ctrl+]` was the default for closing an agent and never once fired**, because
the chord it was written as can never match what arrives. `ctrl+x` replaced it,
and a test now asserts every default is a ctrl **letter**.

**`ctrl+1`..`ctrl+9` is not a thing either**, which is why there is a goto list
instead — see [[Modals]]. tmux ships `extended-keys off` and does not model the
key for `send-keys` either. Inside a modal the keyboard is atrium's, so a plain
digit means what it says.

## Getting a key to the agent

`src/app/input/keys.rs` turns a crossterm `KeyEvent` back into the bytes a
terminal would have sent.

**Key releases are filtered out.** Terminals speaking the kitty keyboard
protocol send press *and* release; without the filter every keystroke would
reach the agent twice.

**Bracketed paste is enabled**, so a paste arrives as one event and is forwarded
as one paste — otherwise the agent sees a burst of individual keystrokes.

## Writing a chord

`ctrl`, `alt` (or `meta`, `opt`, `option`) and `shift`, joined with `+`, then a
key: a single character, `f1`…`f12`, `esc`, `tab`, `space`, `enter`,
`backspace`, the arrows, `home`, `end`, `insert`, `delete`.

`+` itself cannot be bound, which keeps the separator unambiguous. A lone
character is always itself, so a literal `f` is not read as a function key. A
shifted letter is settled to lowercase-plus-SHIFT on both sides before
comparing, because terminals disagree about which they send.
