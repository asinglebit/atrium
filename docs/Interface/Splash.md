# Splash

What atrium shows while it is holding nothing. Guitar's splash, with the
[[Profiles|harnesses]] where guitar lists recent repositories: the profiles you
wrote down, and then whatever CLI atrium knows and found installed that none of
them already names.

```
                  o
        @
P$XR@  P@PP  @BB~ @  #B   @  #BP0@R$0B$
  !v@!  @    @+   @  #B   @  w@   @   @
@   @!  @    @+   @  $@   @  w@   @   @
X@BwP@  w@B  @+   @   @BBw@  w@   @   @
                                      @

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

Three sizes, on guitar's two breakpoints rather than on the widths the art
happens to need — a wordmark reaching the edges of the terminal is not the same
picture as one with room left around it:

| Columns | What is drawn |
| --- | --- |
| 120 and up | The wordmark drawn out, fourteen rows, tail and all |
| 80 to 119 | The same word at half the height, seven rows |
| below 80 | The word `atriuɱ`, whose hooked `m` is that tail kept at one row |

The top 30% take `COLOR_PINK` and the rest `COLOR_PURPLE` — guitar splits its
own logo across two greens the same way. Every size opens on a row that carries
**ink** — the dot of the `i` at the least — so a plain share of the rows puts the
lighter tone on the tops of the letters, which is where it reads.

[[Settings]] is headed with the same small wordmark, on a rule of its own: it
never takes the wide one however wide the terminal, because the version line,
the tab bar and the first setting all have to fit underneath it, and it drops to
the word as soon as its column is narrower than the small wordmark is wide.

## No chrome

The splash is the one view that wears none. No frame, no title line, no status
line, no [[Sidebar]] — nothing is held, so none of them would have anything to
say. It is centred in the bare terminal.

Settings opened over an empty atrium is **not** bare: it is a view of something,
and keeps the frame around it.
