# Keys

**Everything you type reaches the focused agent.** atrium's own actions sit
behind one prefix, `ctrl+space`.

| Gesture | Action |
| --- | --- |
| `ctrl+space` `1`…`9` `0` | the agent on that row, `0` being the tenth |
| `ctrl+space` `space` | go to, which is all of them as a list |
| `ctrl+space` `n` | hold a new agent |
| `ctrl+space` `x` | close this one; closing the last goes back to the [[Splash]] |
| `ctrl+space` `j` / `k` | next / previous |
| `ctrl+space` `shift+1` | show or hide the [[Sidebar]] |
| `ctrl+space` `?` | [[Settings]] |
| `ctrl+space` `q` | quit |

The prefix and all eight actions are rebindable — see [[Configuration]], where
the prefix is the `action` entry. The digits are not: they are the numbers
written down the sidebar, and they mean those rows.

## The digits, and why the sidebar moved

`1`…`9` and then `0` go straight to the agent on that row, which is what the
numbers in the [[Sidebar]] were always for. Ten is as far as one keystroke
goes, so a row past the tenth carries no number and [[Modals|go to]] is how it
is reached.

That is what took `1` off the sidebar, which toggles on `shift+1` now — the
same physical key, with one more finger on it.

A digit is looked at **after** the named actions, so a digit someone has bound
to an action is still that action.

## `ctrl+j` and `ctrl+k` walk a list

Wherever a list or a dialog is up — [[Modals|go to]], the project picker,
[[Settings]], the profile editor, the [[Context menu]], the [[Splash]] — those
two move the cursor. They are turned into the arrows in one place, before
anything else looks at the key, so every list answers them the same way and a
list added later gets them for nothing.

They earn their place in the boxes that take text. The project picker filters
as you type, so a `j` there is a `j`; `ctrl+j` is not, and never was.

## It passes straight through tmux

tmux's prefix here is `C-a`, so `ctrl+space` is nothing tmux wants and reaches
atrium on the first press. `ctrl+a` had to be pressed twice, the second one
arriving only because `bind C-a send-prefix` passed it on.

Nothing else above atrium contends for it either: sway is `Super+…` and ghostty
is `ctrl+shift+…`. **An input method might.** ibus and fcitx take `ctrl+space`
to switch input language by default, and atrium would never see it — rebind
whichever of the two you care less about.

## Why a prefix, and why this one

atrium used to fire eight `ctrl+letter` chords directly. That meant eight keys
an agent could never see, and the choice of defaults was the whole design:
`ctrl+t` cost readline's transpose, `ctrl+n`/`ctrl+p` cost shell history,
`ctrl+x` cost a two-key prefix, and so on. **They are all the agent's again.**

What is left is one key. `ctrl+space` costs readline's set-mark and nothing
else. The keys behind the prefix cost nothing either: they mean something only
in the moment after it.

The letters are guitar's wherever guitar has one, so the two tools do not
disagree: `?` is settings, `x` drops a thing, `q` exits, `j`/`k` walk a list.

A test asserts exactly one chord is claimed, that no action key carries ctrl or
alt, and that the claimed one is none of `ctrl+c`, `ctrl+d`, `ctrl+z`,
`ctrl+v`, `ctrl+l`, `ctrl+r`, `ctrl+u`, `ctrl+w`, `ctrl+e`, `ctrl+k` or
`ctrl+[`, which is Escape.

## A key that means nothing cancels

Press the prefix and then something unbound and nothing happens — the key is
swallowed rather than passed on. Half a mistyped gesture landing in a
conversation is worse than nothing happening. The [[Stage|status line]] shows
`ctrl+space` while atrium waits for the second key, since that is the only
moment a keystroke means something other than itself.

## A binding also has to be deliverable

This is narrower than it looks, and it is the trap this project has fallen into
twice.

atrium does not push the kitty keyboard flags, so it reads the **legacy
encoding**. There, a terminal has a byte for `ctrl` plus a letter and for very
little else:

| What you press | What arrives |
| --- | --- |
| `ctrl+a`…`ctrl+z` | `0x01`…`0x1A` → the letter, with CONTROL |
| `ctrl+space` | `0x00` → **`ctrl+space`**, which is how the prefix can be it |
| `ctrl+]` | `0x1D` → **`ctrl+5`** |
| `ctrl+\`, `ctrl+^`, `ctrl+_` | `0x1C`, `0x1E`, `0x1F` → `ctrl+4`, `ctrl+6`, `ctrl+7` |
| `ctrl+1` | a bare `1` |
| `ctrl+2` | NUL, so it arrives as `ctrl+space` |
| `shift+1` | `!`, with **no** shift flag to read |

**`ctrl+]` was the default for closing an agent and never once fired**, because
the chord it was written as can never match what arrives. `ctrl+x` replaced it,
and a test now asserts every claimed default is a chord that can be delivered:
a ctrl letter, or ctrl and space.

**`ctrl+1`..`ctrl+9` is not a thing either.** That is why the digits sit behind
the prefix as bare keys rather than as chords of their own, and it is why there
is a [[Modals|go to]] list as well. tmux ships `extended-keys off` and does not
model the key for `send-keys` either.

**A shifted digit carries no flag.** The terminal sends `!` and nothing more, so
`!` and `shift+1` are settled onto the one chord before anything is compared,
and either can be written in the config file. They are named for the US layout;
on another the key still works, it is just named after the wrong digit.

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
comparing, and a shifted digit to the digit-plus-SHIFT, because terminals
disagree about the first and say nothing at all about the second.
