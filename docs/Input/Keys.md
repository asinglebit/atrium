# Keys

**Everything you type reaches the focused agent.** `ctrl+space` takes the
keyboard back, and `esc` hands it over again.

It is a **mode, not a prefix**: atrium keeps the keys until you leave, so a run
of them costs one press of the chord rather than one each. Walking the sidebar
and dropping a couple of agents is `ctrl+space` `j` `j` `x` `j` `esc` — one
visit, not five gestures.

| | Who has the keyboard | How you leave |
| --- | --- | --- |
| **Agent**, which is where you start | the focused agent, every keystroke | `ctrl+space` |
| **atrium** | atrium, and the keys below mean what they say | `esc`, or `ctrl+space` again |

While atrium has them the [[Stage|status line]] says **`esc to leave`**, since
that is the half you cannot guess.

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

Written `ctrl+space` `x` throughout, which is how you press it the first time.
Once you are in, `x` on its own is enough.

The chord, the way out and all eight actions are rebindable — see [[Configuration]], where
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

## Why this chord, and why it did not change

atrium used to fire eight `ctrl+letter` chords directly. That meant eight keys
an agent could never see, and the choice of defaults was the whole design:
`ctrl+t` cost readline's transpose, `ctrl+n`/`ctrl+p` cost shell history,
`ctrl+x` cost a two-key prefix, and so on. **They are all the agent's again.**

What is left is one key. `ctrl+space` costs readline's set-mark and nothing
else. The keys behind it cost nothing either: they mean something only while
atrium has the keyboard.

Turning the prefix into a mode did not make a new chord worth looking for. Every
`ctrl`+letter costs the agent that letter outright, and the legacy encoding below
offers nothing else — so the cheapest key there was is still the cheapest key
there is. What changed is how often you press it.

The way out is `esc`, which costs the agent nothing at all: it is read only once
atrium already has the keyboard, so an agent that wants `esc` still gets it
every other moment.

The letters are guitar's wherever guitar has one, so the two tools do not
disagree: `?` is settings, `x` drops a thing, `q` exits, `j`/`k` walk a list.

A test asserts exactly one chord is claimed, that no action key carries ctrl or
alt, and that the claimed one is none of `ctrl+c`, `ctrl+d`, `ctrl+z`,
`ctrl+v`, `ctrl+l`, `ctrl+r`, `ctrl+u`, `ctrl+w`, `ctrl+e`, `ctrl+k` or
`ctrl+[`, which is Escape.

## A key that means nothing is swallowed

Press something unbound while atrium has the keyboard and nothing happens — the
key does not fall through to the agent, and the mode holds. Half a mistyped
gesture landing in a conversation is worse than nothing happening, and being
thrown out by a typo would be worse still.

## Where the mode ends on its own

**A surface opening ends it** — [[Settings]], the [[Modals|picker]], go to, the
[[Context menu]], the profile editor. Each takes the keyboard for itself, and
leaving the mode on underneath would put you back in it on the way out, which is
not where you were going. That holds however the surface was opened, by key or
by mouse.

**Dropping the last agent ends it.** That goes to the [[Splash]], which takes
bare keys of its own.

**On the splash the chord stays a prefix.** atrium already has the keyboard
there, so there is nothing to jump out of: `ctrl+space` `n` reaches the picker
on one press and the mode ends with it, leaving `j`/`k` to the splash's own
list. A mode there could only trap someone whose next key the splash wanted.

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
