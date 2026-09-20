# Worktrees

Somewhere to hold an agent that is not the branch you are standing on.
`src/core/worktree.rs`, and the `worktree` subcommand in `src/main.rs`.

```sh
atrium worktree new fix-42
atrium worktree new fix-42 --in ~/projects/personal/guitar
atrium worktree list
```

Neither opens the TUI. An agent asking for somewhere to work should not have to,
which is the whole reason these exist.

## Where one lands

Beside the repository, named for both, on a branch of the same name:

| Repository | Name | Worktree | Branch |
| --- | --- | --- | --- |
| `~/projects/personal/atrium` | `fix-42` | `~/projects/personal/atrium-fix-42` | `fix-42` |

This is [[Atrium|guitar]]'s convention rather than one of atrium's own, so a
worktree made by either tool looks the same from the other. A name is refused
rather than sanitised when it would change what a path means — empty, `.`, `..`,
or anything holding a separator — because the name becomes both a directory and
a branch.

Worktrees belong to the repository that owns them, which is not the one you are
standing in when you are standing in a worktree. Every operation resolves
through `commondir()` first, so `atrium worktree new` works from inside one.

## A failed worktree takes its branch with it

The branch is made first, then the worktree. If the worktree cannot be made, the
branch is deleted again — a half-made worktree that leaves its branch behind is
exactly the thing you trip over on the next attempt at the same name.

A name that was **already** taken is the other case, and it is left alone: that
branch was never atrium's to roll back.

## Saying so

Having made one, atrium runs whatever `WORKTREE_HOOK` names:

```
$WORKTREE_HOOK created /projects/personal/atrium-fix-42
```

Unset, missing or failing: nothing happens, and that is the ordinary case. Same
policy as [[Adapters and hooks|the hook]] in the other direction — an
announcement is never worth interrupting anything over. The variable is
deliberately not spelled `ATRIUM_*`, because guitar reads the same one and
spells the call the same way, so one hook serves both and neither tool has to
know the other exists.

## Noticing one atrium did not make

On the same three-second tick that re-reads the branch (see [[Registry and focus]]),
atrium looks at the worktrees of the repositories its agents are
standing in, and announces any that have appeared since the last look.

That is the layer that covers guitar with no hook set, a bare `git worktree add`
in a shell, or a script. It only sees repositories an agent is standing in, and
only while atrium is running — which is the most atrium can honestly claim to
know about.

A repository seen for the **first** time announces nothing. Its worktrees were
already there, and announcing all of them at startup is noise rather than news.

## What this does not do

It does not build you a window, switch to one, or know what tmux is. Turning a
worktree into somewhere to work is the job of whatever is listening to the hook.
See [[In tmux]].
