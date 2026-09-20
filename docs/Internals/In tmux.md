# In tmux

What atrium says about itself when it is drawing in a tmux pane, and nothing
else. `src/core/tmux.rs`.

> [!note] atrium still is not a multiplexer
> It drives nothing and depends on nothing. This is the mirror of the way an
> agent calls back into atrium: one thing said, quietly, to whoever is
> listening. Outside tmux none of it happens.

## The two options

Written onto the pane atrium is drawing in, which it finds from `$TMUX_PANE`.
`$TMUX` is checked beside it, so a stale `TMUX_PANE` inherited outside tmux
cannot aim at somebody else's pane.

| Option | Value |
| --- | --- |
| `@atrium_status` | `idle`, `working`, `needs-input` or `error` |
| `@atrium_agents` | `<held> <working> <needs> <error>` |

The status is the **worst** of what the held agents are doing, ordered
`error > needs-input > working > idle`.

## Two spellings of the same status

`needs-input` is hyphenated, where [[Status]] spells the same thing "needs you".
The value is matched against inside a tmux format string, and a space would not
survive the split. The label is for reading; this is for matching.

`Exited` is never published. A row that has finished is not something to be
pulled back to, and an atrium holding nothing but dead agents needs nothing at
all — both options are cleared rather than set to anything.

## When it is said

**Only when it changes.** Saying it every frame would be a process every sixteen
milliseconds. The first frame always says it, which is what makes a fresh atrium
clear a value left in that pane by one that was killed rather than quit, and
quitting clears it on the way out.

Killed with `SIGKILL` atrium clears nothing, and the stale value stands until
something else takes the pane. That is the same trade the [[Adapters and hooks|socket]]
makes with its stale `.sock` files, and it is swept the same way:
by the next thing to come along.

## One invocation, not three

Both options and the repaint go in a single `tmux` call, separated the way tmux
separates commands:

```
set-option -p -t %3 @atrium_status needs-input ; set-option -p ... ; refresh-client -S
```

This runs on the draw thread, so three processes would cost three times what one
does.

The `refresh-client -S` is load-bearing rather than tidy. `status-interval` is a
minute under a default tmuxbar, so without an explicit repaint the colour would
lag by up to that long — long enough to look broken.

## What reads them

Nothing, unless you point something at them. Both are ordinary tmux user
options, readable from a shell with `tmux show-options -p`, or from a format
string. tmuxbar colours each window by the worst thing the atriums in it need,
which it does with `#{P:#{@atrium_status}}` — a walk over the window's own panes,
evaluated as tmux paints, with nothing polling and no daemon anywhere.

See [[Worktrees]] for the other direction: what atrium says when it *makes*
something, rather than what it is doing.
