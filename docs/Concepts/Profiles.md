# Profiles

A **profile** is a named launch recipe: which CLI, plus what to add to its
command line and its environment.

It exists because a Claude **subscription** is nothing more than that. These
were two shell aliases:

```sh
alias clw='CLAUDE_CONFIG_DIR=$HOME/.claude-work claude --append-system-prompt-file "$DOTFILES/…"'
alias clp='CLAUDE_CONFIG_DIR=$HOME/.claude-personal claude --append-system-prompt-file "$DOTFILES/…"'
```

An environment variable and a flag. atrium could reach neither, because an
`AgentSpec` carried only a program, its arguments and a directory.

## Where they live

`~/.config/atrium/profiles.json`, a file **atrium writes**, beside `theme.json`
and `layout.json`. They are not in `config.toml`, and that is deliberate: they
are edited in the [[Settings]] view, and rewriting a TOML file drops its
comments and reformats everything else in it. A `config.toml` still carrying the
old `[[profiles]]` block is told where they went — see [[Configuration]].

```json
{
  "default": "work",
  "profiles": [
    { "name": "work", "config_dir": "~/.claude-work",
      "args": ["--append-system-prompt-file", "$DOTFILES/shared/prompts/system-prompt.md"] }
  ]
}
```

| Field | Meaning |
| --- | --- |
| `name` | Required. What the picker and the [[Sidebar]] row call it |
| `program` | The CLI. Empty means `claude`, so a subscription need not say it |
| `config_dir` | Shorthand for `CLAUDE_CONFIG_DIR` — which subscription, and nothing else |
| `args` | Added to the command line, in order |
| `env` | `[{ "name": …, "value": … }]`, for anything `config_dir` does not cover |

**What is stored is raw.** `~/.claude-work` stays `~/.claude-work` on disk and
is expanded only on the way to a child process. Saving the expanded form would
quietly hardcode one machine's home directory into a file you might sync.

An unset variable expands to nothing, the way a shell would; `$5` is five
dollars, because a name has to start with a letter or an underscore. The rules
live in `expand_with`, which takes its lookup as an argument — not decoration:
setting an environment variable is process-wide and the test suite runs in
parallel, so a test that set one to check expansion would race every other test
that read one.

## Managing them

The `profiles` tab in [[Settings]], modelled on guitar's remote management.

```
 profiles:

 actions:            select to manage | + add to create

 + add profile                                  (enter)
 work       ~/.claude-work                          🞊
 personal   ~/.claude-personal                      🞅
```

**`+ add profile`** chains three prompts — name, then config dir prefilled with
`~/.claude-<name>`, then args — so the common case is a name and two presses of
enter. Esc backs out of the whole thing rather than one step, because half an
added profile is not worth keeping.

**Selecting one** opens an action list: `set as default`, `rename`,
`edit config dir`, `edit args`, `delete`. Editing prefills with what is there
now, so it is a correction rather than retyping. Deleting asks first.

Every change is written straight away and taken up at once — the next `ctrl+t`
sees it without atrium being restarted. A refusal, such as a name already taken,
stays in the modal so the answer can be corrected without starting again.

Args are typed as **one line and split on spaces**, which cannot carry a quoted
argument containing one. The prompt says so rather than pretending otherwise.

## Choosing one

- **`atrium`** holds whatever `default` names. An unknown default falls back to
  the first profile.
- **`atrium --profile personal`** holds that one instead — the flag the two
  aliases collapse into.
- **The [[Splash]]**, which is what a bare `atrium` opens on: the profiles are
  its list, and enter holds the selected one where you started.
- **`ctrl+t`**, then `tab`. The picker's `tab` used to cycle bare CLI names; it
  now cycles profiles, so the axis that already existed does the job and the
  modal gained no new key. See [[Modals]].

## What is installed

atrium also looks on `PATH` for the CLIs it knows — `claude`, `opencode`,
`codex` — and offers each one it finds as a profile of its own, so the picker
has a single code path and a machine with nothing written down still has
something to hold. `src/core/installed.rs`.

An installed CLI is offered **unless a profile already names it**. Two claude
subscriptions are how you hold claude; a bare `claude` beside them would be a
third way of saying the same thing. So `work` and `personal` plus an installed
opencode is three rows, not four — and deleting the last profile leaves what is
installed rather than nothing.

One that is **not** installed is not offered, because picking it is a launch
that can only fail. The scan happens once at start, and what it found is listed
in the [[Settings]] profiles tab, each with the path it was found at — a `codex`
missing from the list is a question the tab answers.

A CLI found this way is never written to `profiles.json`. It was not configured;
it was noticed.

## Seeing which one

A profile whose name *is* its program renders as just `claude` and tags no
[[Sidebar]] row: every row would carry the same word, which says nothing. A
named one shows both — `claude · work` in the picker, `work` on the row.

That tag is the point. Two agents on the same project under different
subscriptions are otherwise the same row twice.

## What it does not touch

- `atrium bash --norc` picks up **no** profile. A command named on the command
  line is held exactly as written.
- Hooks are unaffected — they go in via `--settings` (see
  [[Adapters and hooks]]), independent of `CLAUDE_CONFIG_DIR`.
- A profile's env is applied **before** the adapter's own wiring, so it cannot
  shadow `ATRIUM_AGENT_ID` or `ATRIUM_SOCK`.
