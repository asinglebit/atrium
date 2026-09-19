# Splash

What atrium shows while it is holding nothing. Guitar's splash, with the
[[Profiles|harnesses]] where guitar lists recent repositories: the profiles you
wrote down, and then whatever CLI atrium knows and found installed that none of
them already names.

```
              68b
 /            Y89
 ___   /M     ___  __ ___ ___   ___ ___  __    __
6MMMMb /MMMMM  `MM 6MM `MM `MM    MM `MM 6MMb  6MMb
…
`YMMM9'Yb.YMMM9 _MM_    _MM_  YMMM9MM__MM_  _MM_  _MM_

                      harnesses

  actions: enter holds one here | ctrl+t pick a project | ctrl+s settings

                  ⏵ claude · work ⏴
                   claude · personal
                        opencode
```

## When it appears

- **`atrium`**, with nothing to go on. Nothing was asked for, so it asks.
- **After the last agent closes.** `ctrl+x` on the only held agent returns here
  rather than ending atrium — see [[Registry and focus]]. `ctrl+q` is how you
  leave.

`atrium --profile personal` and `atrium bash --norc` **skip it**: both already
said what they wanted.

## Using it

`j`/`k` and the arrows move, `enter` holds the selected harness **in the
directory atrium was started in** — which is what a bare `atrium` used to do
without asking. A click holds the row it landed on, because the list exists to
be picked from and selecting then confirming would be two steps for one
intention.

The chords still work: `ctrl+t` opens the [[Modals|project picker]] instead,
`ctrl+s` opens [[Settings]], `ctrl+q` quits.

A launch that fails — a profile whose program has been uninstalled since the
scan — leaves its message above the list rather than taking atrium down, and the
list shifts down to make room for it. The [[Modals|picker]] keeps a failed launch
the same way.

A machine with nothing installed and nothing configured has an **empty list**,
and the splash says so in place of it, naming the three CLIs it looked for.
Offering a `codex` that is not there would only produce that failure line.

## The wordmark

Three sizes, the largest that fits, the way guitar's scales:

| Columns | What is drawn |
| --- | --- |
| 54 and up | The full wordmark, eleven rows |
| 25 to 53 | The block, five rows — the same one [[Settings]] is headed with |
| below 25 | The word `atrium` |

The top 30% take `COLOR_PINK` and the rest `COLOR_PURPLE` — guitar splits its
own logo across two greens the same way. The share is counted over the rows that
carry **ink**: the wide wordmark opens with a blank row, and counting it would
leave the lighter tone on the dot of the `i` and nothing else.

## No chrome

The splash is the one view that wears none. No frame, no title line, no status
line, no [[Sidebar]] — nothing is held, so none of them would have anything to
say. It is centred in the bare terminal.

Settings opened over an empty atrium is **not** bare: it is a view of something,
and keeps the frame around it.
