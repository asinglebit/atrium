# Profiles

A **profile** is a named launch recipe: which CLI, plus what to add to its
command line and its environment. `src/core/profile.rs`.

It exists because a Claude **subscription** is nothing more than that. These
were two shell aliases:

```sh
alias clw='CLAUDE_CONFIG_DIR=$HOME/.claude-work claude --append-system-prompt-file "$DOTFILES/…"'
alias clp='CLAUDE_CONFIG_DIR=$HOME/.claude-personal claude --append-system-prompt-file "$DOTFILES/…"'
```

An environment variable and a flag. atrium could reach neither, because an
`AgentSpec` carried only a program, its arguments and a directory — so holding a
personal-subscription agent inside atrium was impossible.

## Writing one

```toml
default = "work"

[[profiles]]
name = "work"
config_dir = "~/.claude-work"
args = ["--append-system-prompt-file", "$DOTFILES/shared/prompts/system-prompt.md"]
```

| Setting | Meaning |
| --- | --- |
| `name` | Required. What the picker and the row call it |
| `program` | The CLI. Defaults to `claude`, so a subscription need not repeat it |
| `config_dir` | Shorthand for `CLAUDE_CONFIG_DIR` — which subscription, and nothing else |
| `args` | Added to the command line, in order |
| `env` | An inline table, for anything `config_dir` does not cover |

`config_dir` and every `args` entry get **`~` and `$VAR` expansion**, so a path
can be pasted straight out of a shell alias rather than hardcoded. An unset
variable expands to nothing, the way a shell would; `$5` is five dollars,
because a name has to start with a letter or an underscore.

The rules live in `expand_with`, which takes its lookup as an argument. That is
not decoration: setting an environment variable is process-wide and the test
suite runs in parallel, so a test that set one to check expansion would race
every other test that read one.

## Choosing one

- **`atrium`** holds whatever `default` names. An unknown `default` is reported
  by [[Configuration|--check-config]] and the first profile is used.
- **`atrium --profile personal`** holds that one instead. This is the flag the
  two aliases collapse into.
- **`ctrl+t`**, then `tab`. The picker's `tab` used to cycle bare CLI names; it
  now cycles profiles, so the axis that already existed does the job and the
  modal gained no new key. See [[Modals]].

With no `[[profiles]]` configured at all, atrium synthesises one per CLI it
knows — `claude`, `opencode`, `codex` — so the picker has a single code path and
nothing changes for someone who has never written a profile.

## Seeing which one

A profile whose name *is* its program renders as just `claude`, and tags no
[[Sidebar]] row: every row would carry the same word, which says nothing. A
named one shows both — `claude · work` in the picker, `work` on the row.

That tag is the point. Two agents on the same project under different
subscriptions are otherwise the same row twice.

## What it does not touch

- `atrium bash --norc` picks up **no** profile. A command named on the command
  line is held exactly as written, rather than inheriting Claude's environment.
- Hooks are unaffected — they go in via `--settings` (see
  [[Adapters and hooks]]), independent of `CLAUDE_CONFIG_DIR`.
- A profile's env is applied **before** the adapter's own wiring, so a profile
  cannot shadow `ATRIUM_AGENT_ID` or `ATRIUM_SOCK`.
