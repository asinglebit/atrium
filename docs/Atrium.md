# Atrium

A terminal UI that **holds coding agents**. It spawns agent CLIs as its own
children, keeps them in a sidebar, and shows what each one is doing.

> [!warning] Very much a work in progress
> Built as it is used, so the shape moves week to week. Bindings, config files
> and layout are all fair game to change. These notes describe what is there
> today, not what is promised.

Same family as **guitar** — same chrome, same palette, same `theme.json` —
pointed at a different job. guitar reads a repository; atrium runs the agents
that change one. Both are written for one person's day-to-day work first, which
is why the defaults are opinionated and the scope stops where that work stops.

## Contents

### Concepts

- [[Agents]] — what atrium holds, and how it holds it
- [[Registry and focus]] — the held set, and which one is on the stage
- [[Status]] — the five states, where they come from, what they look like
- [[Profiles]] — subscriptions, and anything else a CLI needs to be launched with
- [[Projects]] — how the project list is found
- [[Worktrees]] — cutting somewhere new to hold an agent, and saying so

### Interface

- [[Splash]] — what atrium shows while it holds nothing
- [[Sidebar]] — the rows, the widths, the scrollbar that is also a divider
- [[Stage]] — the embedded terminal the agent draws on
- [[Settings]] — the themes and shortcuts view
- [[Context menu]] — right-click, anywhere
- [[Modals]] — the new-agent picker and the goto list

### Input

- [[Keys]] — every binding, and why each one costs what it costs
- [[Mouse]] — click, wheel, drag, right-click, and what reaches the agent

### Configuration

- [[Configuration]] — the three files, and which of them you write
- [[Themes]] — 47 presets, shared with guitar

### Internals

- [[Adapters and hooks]] — how an agent reports what it is doing
- [[Layout]] — where everything goes, and what happens when it does not fit
- [[In tmux]] — the one thing atrium says about itself, and to whom
- [[Code map]] — the module tree and the test convention

## The shape of it

```
  atriuɱ |  ~/projects/personal/atrium                        agents
╭──────────────────────────────────────────────────────────────────╮
│ ⠙ 1 atrium               master* │ the focused agent's own       │
│ ● 2 guitar                  main │ terminal, drawn as it drew    │
│ ○ 3 bazzite                 sway │ itself                        │
╰──────────────────────────────────────────────────────────────────╯
  guitar working  ● main                                        2/3
```

Title line above, status line below, a rounded frame around the two panes. The
column between the panes is both the divider and the sidebar's scrollbar.
