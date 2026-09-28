# In tmux

What atrium says about itself when it is drawing in a tmux pane, and nothing
else. `src/core/tmux.rs`.

> [!note] atrium still is not a multiplexer
> It drives nothing and depends on nothing. This is the mirror of the way an
> agent calls back into atrium: one thing said, quietly, to whoever is
> listening. Outside tmux none of it happens.

## The options

Written onto the pane atrium is drawing in, which it finds from `$TMUX_PANE`.
`$TMUX` is checked beside it, so a stale `TMUX_PANE` inherited outside tmux
cannot aim at somebody else's pane.

| Option | Scope | Value |
| --- | --- | --- |
| `@atrium_status` | pane | `idle`, `working`, `needs-input`, `done` or `error` |
| `@atrium_agents` | pane | `<held> <working> <needs> <error>` |
| `@atrium_unseen` | pane | `done` or `error`: how the last turn to end here ended, while nobody was looking |
| `@atrium_blink` | server | `1` or `0`, the half of the flicker to draw |

The status is the **worst** of what the held agents are doing, ordered
`needs-input > working > error > done > idle`. Waiting on you outranks
everything, and still going outranks how a turn ended: whether an ending has
been seen is `@atrium_unseen`'s to say, so the status only has to tell a window
whether anything is waiting or working.

## Two spellings of the same status

`needs-input` is hyphenated, where [[Status]] spells the same thing "needs you".
The value is matched against inside a tmux format string, and a space would not
survive the split. The label is for reading; this is for matching.

`Exited` is never published. A row that has finished is not something to be
pulled back to, and an atrium holding nothing but dead agents needs nothing at
all — the options are cleared rather than set to anything.

## Unseen, until the window is visited

A turn that ends -- `Done` or `Error` -- marks the pane `@atrium_unseen`, and
tmuxbar paints the window green or red for as long as the mark stands. Visiting
the window is what takes it away: tmuxbar hooks `session-window-changed`,
`client-session-changed` and `client-attached`, and landing on a window by any
of them unmarks every pane in it.

It is set **only at the moment a turn ends**, and never again. The status is
re-said whenever anything changes; the mark is not, so a visit's clearing has
the last word until the next turn ends.

It is set **only if nobody is looking**. The check runs inside tmux, as an
`if-shell -F` on `#{window_active_clients}` in the same invocation as the write,
so a turn that ends in front of you is seen rather than announced. A failure
already waiting to be seen is not covered by a finish.

**Pane options, not tmux's bell flag.** A bell would have been the obvious
signal: tmux already marks a window that rang until it is visited, and never
marks the one you are on. But the flag is per session, and a group of sessions
copies it among themselves whenever a window is created in one -- so with a
session per terminal, a window you watched finish lit up again the next time a
terminal opened, and selecting it again did not clear it. A pane option belongs
to the pane, which every session in the group shares: a visit from any terminal
counts for all of them.

## Keeping the beat

`@atrium_blink` is the one thing here said on a timer rather than on a change,
and the one thing set on the server rather than on the pane: every window that
holds a working agent beats together, and a window format finds a global option
by walking up from the pane.

It exists because nothing else can keep that time. tmux repaints its status line
only when asked, and the terminal's own blink attribute is ignored by ghostty —
so a window cannot flicker unless something asks for a repaint twice a second,
and only an atrium knows there is an agent still working.

So the beat runs **only while this atrium's published status is `working`**,
which is exactly the status tmuxbar draws flickering. Nothing working, nothing
ticking: an atrium holding finished or waiting agents costs a tmux invocation
only when something changes.

It always stops **lit**. tmuxbar dims on an explicit `0` and lights on anything
else — `1`, and an option never set at all — so a beat that ends mid-cycle,
or an atrium killed before it could stop cleanly, leaves a window that has
stopped moving rather than one stuck on its darker half.

The phase is read off the wall clock, so two atriums on one server write the
same value at the same moment instead of fighting over it with two rhythms.

## When it is said

**Only when it changes.** Saying it every frame would be a process every sixteen
milliseconds. The first frame always says it, which is what makes a fresh atrium
clear a value left in that pane by one that was killed rather than quit, and
going away clears all of it on the way out, the unseen mark included.

A turn ending is said even when the worst status has not moved: one agent
finishing beside another still working changes nothing else.

Clearing is in `Drop` rather than at the end of the draw loop, so it happens
whatever ends the loop — a quit, an error out of a `?`, or a panic unwinding
through it. Only a signal gets past it: killed, atrium clears nothing, and the
stale value stands until something else takes the pane. A pane that closes takes
its own options with it, so what this leaves open is an atrium killed in a pane
you keep. That is the same trade the [[Adapters and hooks|socket]] makes with
its stale `.sock` files, and it is swept the same way: by the next thing to come
along.

## One invocation, not three

The options, the unseen mark and the repaint go in a single `tmux` call,
separated the way tmux separates commands:

```
set-option -p -t %3 @atrium_status done ; set-option -p ... ; if-shell -F -t %3 '#{==:#{window_active_clients},0}' 'set-option -p -t %3 @atrium_unseen done' ; refresh-client -S
```

This runs on the draw thread, so three processes would cost three times what one
does.

The `refresh-client -S` is load-bearing rather than tidy. `status-interval` is a
minute under a default tmuxbar, so without an explicit repaint the colour would
lag by up to that long — long enough to look broken.

## What reads them

Nothing, unless you point something at them. They are ordinary tmux user
options, readable from a shell with `tmux show-options -p`, or from a format
string. tmuxbar colours each window with `#{P:#{@atrium_status}}` and
`#{P:#{@atrium_unseen}}` — walks over the window's own panes, evaluated as tmux
paints, with nothing polling and no daemon anywhere.

See [[Worktrees]] for the other direction: what atrium says when it *makes*
something, rather than what it is doing.
