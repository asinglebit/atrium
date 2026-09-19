# Keys

Everything you type goes to the focused agent, except these eight. **Actions
fire directly — there is no leader to press first.**

| Chord | Action |
| --- | --- |
| `ctrl+t` | hold a new agent |
| `ctrl+x` | close this one; closing the last ends atrium |
| `ctrl+n` / `ctrl+p` | next / previous |
| `ctrl+g` | go to, where `1`…`9` jump straight to a row |
| `ctrl+o` | show or hide the [[Sidebar]] |
| `ctrl+s` | [[Settings]] |
| `ctrl+q` | quit |

All eight are rebindable — see [[Configuration]].

## Why these

No leader means every binding is a key **taken from the agent**, which makes the
choice of defaults the whole design. `ctrl+letter` is a crowded space, and
anything atrium claims the agent never sees. So the defaults are picked for what
they cost, not for the mnemonic:

- **`ctrl+q`, `ctrl+s`** — XON and XOFF. Raw mode has already turned flow
  control off, so neither can freeze anything.
- **`ctrl+n`, `ctrl+p`** — shell history, which Claude's own input box does not
  use; it uses the arrows.
- **`ctrl+t`** — readline's transpose.
- **`ctrl+g`** — aborts a readline entry, rarely asked for on purpose.
- **`ctrl+o`** — readline's operate-and-get-next, which nothing asks for.
- **`ctrl+x`** — a two-key readline prefix, and an agent's input box implements
  no second key to follow it.

**Deliberately untouched**, so the agent keeps them: `ctrl+c`, `ctrl+d`,
`ctrl+z`, `ctrl+v`, `ctrl+l`, `ctrl+r`, `ctrl+u`, `ctrl+w`, `ctrl+a`, `ctrl+e`,
`ctrl+k`, and `ctrl+[` which is Escape. A test asserts no default lands on one
of them.

Nothing above atrium contends for these either: sway is `Super+…`, ghostty is
`ctrl+shift+…`, and tmux claims `C-a` plus thirteen prefix-less `M-` bindings —
no plain `ctrl+letter` among them. The one to avoid is `ctrl+a`, which tmux
takes as its prefix and atrium would therefore never receive.

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
