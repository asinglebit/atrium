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

  enter holds one here | ctrl+space n pick a project | ctrl+space ? settings

                  ⏵ claude · work ⏴
                   claude · personal
                        opencode
```

## When it appears

- **`atrium`**, with nothing to go on. Nothing was asked for, so it asks.
- **After the last agent closes.** `ctrl+space` `x` on the only held agent
  returns here rather than ending atrium — see [[Registry and focus]].
  `ctrl+space` `q` is how you leave.

`atrium --profile personal` and `atrium bash --norc` **skip it**: both already
said what they wanted.

## Using it

`j`/`k`, `ctrl+j`/`ctrl+k` and the arrows move, `enter` holds the selected
harness **in the
directory atrium was started in** — which is what a bare `atrium` used to do
without asking. A click holds the row it landed on, because the list exists to
be picked from and selecting then confirming would be two steps for one
intention.

The gestures still work: `ctrl+space` `n` opens the [[Modals|project picker]]
instead, `ctrl+space` `?` opens [[Settings]], `ctrl+space` `q` quits.

A launch that fails — a profile whose program has been uninstalled since the
scan — leaves its message above the list rather than taking atrium down, and the
list shifts down to make room for it. The [[Modals|picker]] keeps a failed launch
the same way.

A machine with nothing installed and nothing configured has an **empty list**,
and the splash says so in place of it, naming the CLIs it looked for on a line
of their own -- with the sentence they had outgrown eighty columns.
Offering a `codex` that is not there would only produce that failure line.

## The wordmark

Three sizes, on guitar's two breakpoints rather than on the widths the art
happens to need — a wordmark reaching the edges of the terminal is not the same
picture as one with room left around it:

| Columns | What is drawn |
| --- | --- |
| 106 and up | The wordmark drawn out, fourteen rows, tail and all |
| 80 to 105 | The same word at half the height, seven rows |
| below 80 | The word `atriuɱ`, whose hooked `m` is that tail kept at one row |

The top 30% take the lighter of two purples and the rest the darker — guitar
splits its own logo across two greens the same way. Every size opens on a row
that carries **ink** — the dot of the `i` at the least — so a plain share of the
rows puts the lighter tone on the tops of the letters, which is where it reads.

[[Settings]] is headed with the same small wordmark, on a rule of its own: it
never takes the wide one however wide the terminal, because the version line,
the tab bar and the first setting all have to fit underneath it, and it drops to
the word as soon as its column is narrower than the small wordmark is wide.

## It moves

**At rest the wordmark is the art exactly as it was drawn.** Everything below
starts from there, so what atrium shows in its first instant is the picture, and
what the animation does to it is something you can always name.

Two things happen, both read off how long atrium has been up rather than kept
anywhere — the same way the [[Status|spinner]] and the pulse are driven, because
the draw loop already runs sixty times a second.

**A grain.** Characters drift. A cell only ever swaps for another glyph the art
itself draws, and only inside its own weight — `@#$BR`, `0XPw`, `covI!+`, `.:~"`
— so the letterforms hold and the word is never a smear. About one cell in
eighteen is carrying a borrowed glyph at any moment, each on its own beat, so
the grain crawls rather than blinking. `atriuɱ` is left alone: its letters are
in none of those groups, and a shuffled one would spell something else.

**A sheen.** A band of light crosses the wordmark every five seconds or so,
leaning two columns a row. The crest is cut into one step per entry in its
**profile**, and a cell is carried along the whole of it as the light passes:

```
rest  ->  lightest  ->  rest  ->  darkest  ->  rest
      0 -1 -2 -3 -2 -1  0  1  2  3  2  1  0
```

So the band reads **light at its left and dark at its right** — the ramp written
along the wordmark the way you read it — and it **eases to nothing at both
ends** rather than meeting the resting colour on a hard line. The tail after the
dark section is measured along the same diagonal as everything else, so it leans
with the rest of the sheen.

The ramp is six stops of **one purple**, from its lightest to its darkest. It is
mixed out of `COLOR_PURPLE` rather than taken from the palette, because there is
no light or dark purple in there to take: `COLOR_PURPLE` and `COLOR_DURPLE`
differ in hue, not in lightness, so a ramp of the two would hardly move. Moving
in lightness alone is what makes it read as light crossing one colour rather
than a run through several.

The wordmark rests on the middle two stops — a step lighter than the palette's
purple for the top rows, a step darker for the rest — and the light carries a
cell out to either end. It also stirs the grain where it passes, up to one cell
in four, and falls back to nothing before it wraps, so the sweep never jumps.
Every size gets the same picture at its own scale, and the same five seconds, so
the small wordmark does not race the big one.

There is no switch for it, as there is none for the spinner or the pulse.

## No chrome

The splash is the one view that wears none. No frame, no title line, no status
line, no [[Sidebar]] — nothing is held, so none of them would have anything to
say. It is centred in the bare terminal.

Settings opened over an empty atrium is **not** bare: it is a view of something,
and keeps the frame around it.
