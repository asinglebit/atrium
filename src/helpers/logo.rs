use std::time::Duration;

use ratatui::{
    style::{Color, Style},
    text::{Line, Span},
};

use crate::helpers::palette::{Theme, blend, distinct};

/// atrium's wordmark at its largest, for the splash on a terminal with the
/// width to give it. The tail the `m` hangs on is the one the word ends on at
/// every size, down to the `ɱ` of `COMPACT`.
pub const WIDE: [&str; 14] = [
    r##"                                  .~.                                      "##,
    r##"                                 .0@$                                      "##,
    r##"                0@"                :                                       "##,
    r##"   .!++~       +R@+"""   +~  "+~  .""    "".     .""    +"  .!+~    ~++.   "##,
    r##" I$#R#BR$$    $R@@RRRR   BRP$#Bw  vRR    #Rc     IRR    BRX$$PPR$P$$PP##$c "##,
    r##"       .0@B    v@@"      0@@+     I@@    @@o     o@@    o@@    "#@$    !R@I"##,
    r##"    ~:co$@B    v@@"      o@B"     I@@    @@o     o@@    I@@     c@@     o@B"##,
    r##" X@@@$o:0@B    v@@"      o@B"     I@@    @@o     o@@    I@@     c@@     o@B"##,
    r##"v@@"    o@B    v@@"      o@B"     I@@    R@0     o@@    I@@     c@@     o@B"##,
    r##"v@@o   :R@B"   !R@0      o@B"     I@@    v@B~   "$@@    I@@     c@@     o@B"##,
    r##" v$$$$$I 0$o    +$$$$$+  c$P"     v$$     c$$$$$XvP$.   v$$     :$$     o@B"##,
    r##"                                                                        o@B"##,
    r##"                                                                       ~#@#"##,
    r##"                                                                     cwwwv "##,
];

pub const WIDE_WIDTH: usize = 75;

/// The same word drawn smaller, for the splash on a terminal without that
/// width. Half the height and half again the width, tail included.
pub const NARROW: [&str; 7] = [
    r##"                  o                    "##,
    r##"        @                              "##,
    r##"P$XR@  P@PP  @BB~ @  #B   @  #BP0@R$0B$"##,
    r##"  !v@!  @    @+   @  #B   @  w@   @   @"##,
    r##"@   @!  @    @+   @  $@   @  w@   @   @"##,
    r##"X@BwP@  w@B  @+   @   @BBw@  w@   @   @"##,
    r##"                                      @"##,
];

pub const NARROW_WIDTH: usize = 39;

/// What a column with room for none of them gets instead. The hooked `ɱ` is the
/// drawn wordmark's tail kept at one row -- guitar ends its own compact logo on
/// a corner glyph the same way.
pub const COMPACT: &str = "atriuɱ";

/// Where the splash changes wordmark. Guitar's two breakpoints, not the widths
/// the art happens to need: a wordmark reaching the edges of the terminal is
/// not the same picture as one with room left around it.
const WIDE_COLUMNS: usize = 106;
const NARROW_COLUMNS: usize = 80;

/// How many of `total` rows take the lighter tone: the top 30%, rounded up so a
/// short wordmark still gets some, and never fewer than one.
pub fn bright_rows(total: usize) -> usize {
    if total == 0 { 0 } else { (total * 3).div_ceil(10).max(1) }
}

/// The smaller wordmark or the word. What the settings header uses, which never
/// takes the wide one however much width it has: the version line, the tab bar
/// and the first setting all have to fit underneath it.
pub fn rows_for(width: usize) -> &'static [&'static str] {
    if width >= NARROW_WIDTH { &NARROW } else { std::slice::from_ref(&COMPACT) }
}

/// The biggest the terminal has room for. What the splash uses, where the
/// wordmark is the point.
pub fn splash_rows_for(width: usize) -> &'static [&'static str] {
    if width >= WIDE_COLUMNS {
        &WIDE
    } else if width >= NARROW_COLUMNS {
        &NARROW
    } else {
        std::slice::from_ref(&COMPACT)
    }
}

/// Glyphs of like weight, so a swap inside a group keeps the letterform. The
/// wordmark only ever borrows from its own alphabet -- the four groups together
/// are every glyph the art draws -- which is why the `atriuɱ` of `COMPACT` is
/// left alone: none of its letters are in here.
const GRAINS: [&str; 4] = ["@#$BR", "0XPw", "covI!+", ".:~\""];

/// How long a cell keeps a glyph before it may take another.
const HOLD_MS: u128 = 140;

/// One cell in how many carries a swapped glyph: away from the light, at the
/// edges of the wave, and through its middle. The wave stirs the grain where it
/// passes and leaves it alone behind.
const DENSITY: [u64; 3] = [18, 7, 4];

/// How far the wave leans: two columns a row, which is about forty-five degrees
/// once the terminal's cells are twice as tall as they are wide.
const LEAN: usize = 2;

/// How far along the ramp the light carries a cell, step by step across the crest and read left to
/// right: out to the lightest tint, down through the ramp to the darkest, and back to where the
/// wordmark rests. Easing to nothing at both ends is what keeps the sheen from meeting the resting
/// colour on a hard line, and every step is measured along the same diagonal, so the tail leans
/// with the rest of it.
const PROFILE: [isize; 13] = [0, -1, -2, -3, -2, -1, 0, 1, 2, 3, 2, 1, 0];

/// How long one pass takes, crest and the quiet after it. The same at every
/// size, so the small wordmark does not sweep faster than the big one.
const SWEEP_MS: u128 = 5200;

/// A number that looks random for a cell but is the same every time it is asked
/// for, so the animation keeps no state of its own.
fn scatter(row: usize, column: usize, tick: u64) -> u64 {
    let mut seed = (row as u64).wrapping_mul(0x9E37_79B9_7F4A_7C15) ^ (column as u64).wrapping_mul(0xC2B2_AE3D_27D4_EB4F) ^ tick.wrapping_mul(0x1656_67B1_9E37_79F9);
    seed ^= seed >> 30;
    seed = seed.wrapping_mul(0xBF58_476D_1CE4_E5B9);
    seed ^= seed >> 27;
    seed = seed.wrapping_mul(0x94D0_49BB_1331_11EB);
    seed ^ (seed >> 31)
}

/// Which of a cell's holds is current. Every cell keeps its own offset into the
/// beat, so the swapped glyphs turn over one at a time rather than the whole
/// wordmark blinking at once.
fn hold(row: usize, column: usize, elapsed: Duration) -> u64 {
    let offset = scatter(row, column, 0) % HOLD_MS as u64;
    ((elapsed.as_millis() + offset as u128) / HOLD_MS) as u64
}

/// What a cell draws now. A glyph the art does not use -- a blank, a letter of
/// the word -- is left as it is, and so is every glyph before the first beat:
/// the wordmark arrives as it was drawn and then wakes up.
fn grain(glyph: char, row: usize, column: usize, density: u64, elapsed: Duration) -> char {
    let Some(group) = GRAINS.iter().find(|group| group.contains(glyph)) else {
        return glyph;
    };
    let hold = hold(row, column, elapsed);
    if hold == 0 {
        return glyph;
    }
    let roll = scatter(row, column, hold);
    if !roll.is_multiple_of(density) {
        return glyph;
    }
    group.as_bytes()[(roll >> 8) as usize % group.len()] as char
}

/// The band of light travelling over the wordmark. Its size falls out of the
/// art's own, so every wordmark gets the same picture at its own scale.
struct Sweep {
    /// The travel plus a quiet gap, so nothing is lit at either end of the
    /// cycle and the wrap back to the start does not jump.
    period: usize,
    /// How wide the crest is, measured along its travel.
    crest: usize,
}

impl Sweep {
    fn over(rows: &[&str]) -> Self {
        let span = rows.first().map_or(0, |row| row.chars().count()) + LEAN * rows.len();
        let crest = (span / 4).max(PROFILE.len());
        Self { period: span * 4 / 3 + crest, crest }
    }

    /// Where in the light this cell is standing: `None` clear of it, otherwise a
    /// band from 0 at the crest's trailing edge to its last at the leading one.
    /// The crest is cut into one step per entry in the profile, so a cell is
    /// carried along the whole of it as the light passes over.
    fn band(&self, row: usize, column: usize, elapsed: Duration) -> Option<usize> {
        let front = (elapsed.as_millis() * self.period as u128 / SWEEP_MS) as usize % self.period;
        // Zero is the crest still short of this cell, which is where every
        // cycle starts -- so a wordmark at rest is the wordmark as it was drawn.
        let behind = front.saturating_sub(column + row * LEAN);
        if behind == 0 || behind > self.crest {
            return None;
        }
        // Counted back from the far end, so the ramp reads along the wordmark the way it is
        // written: the light leaves its first tint behind it and carries its last at the front.
        Some(PROFILE.len() - 1 - (behind - 1) * PROFILE.len() / self.crest)
    }
}

/// How sparse the grain is where this cell stands. The wave stirs it hardest
/// through the middle of the crest and least out in the dark.
fn stir(band: Option<usize>) -> u64 {
    match band {
        None => DENSITY[0],
        Some(step) if PROFILE[step].abs() <= 1 => DENSITY[1],
        Some(_) => DENSITY[2],
    }
}

/// What the two ends of the ramp are mixed toward, to reach a purple lighter and
/// one darker than the palette carries.
const HIGHLIGHT: Color = Color::Rgb(255, 255, 255);
const SHADOW: Color = Color::Rgb(0, 0, 0);

/// How far the ramp reaches either side of the wordmark's own purple.
const REACH: f32 = 0.45;

/// The ramp the wordmark is lit from: the palette's purple, spread from its
/// lightest to its darkest. It moves in lightness alone, so every stop is the
/// same purple and the wave reads as light crossing one colour rather than a
/// run through several.
///
/// It is mixed rather than taken from the palette because there is no light or
/// dark purple in there to take -- `COLOR_PURPLE` and `COLOR_DURPLE` differ in
/// hue, not in lightness, so a ramp of the two would hardly move.
const TINTS: usize = 6;

fn tints(theme: &Theme) -> [Color; TINTS] {
    let mut ramp = [theme.COLOR_PURPLE; TINTS];
    for (index, tint) in ramp.iter_mut().enumerate() {
        // Evenly spaced from the lightest stop down to the darkest.
        let reach = REACH - 2.0 * REACH * index as f32 / (TINTS - 1) as f32;
        *tint = blend(theme.COLOR_PURPLE, if reach >= 0.0 { HIGHLIGHT } else { SHADOW }, reach.abs());
    }
    distinct(ramp)
}

/// Where the wordmark rests: a step lighter than the palette's purple for the
/// top rows, a step darker for the rest.
const RESTING: usize = 2;

/// What a row is lit in once the wave has pushed it this far. Every tone comes
/// from the palette, so a retheme carries the wordmark with it, and living here
/// is what keeps the splash and the settings header from drifting apart.
///
/// Every wordmark opens on a row that carries ink -- the dot of the `i` at the
/// least -- so a plain share of the rows puts the lighter tone on the tops of
/// the letters, which is where it reads. The wave rides on that split rather
/// than replacing it.
fn tone(row: usize, band: Option<usize>, rows: &[&str], tints: &[Color; TINTS]) -> Color {
    let resting = (RESTING + usize::from(row >= bright_rows(rows.len()))) as isize;
    let rolled = band.map_or(0, |step| PROFILE[step]);
    tints[(resting + rolled).clamp(0, TINTS as isize - 1) as usize]
}

/// What a cell is lit in at this moment.
pub fn tint(row: usize, column: usize, rows: &[&str], theme: &Theme, elapsed: Duration) -> Color {
    tone(row, Sweep::over(rows).band(row, column, elapsed), rows, &tints(theme))
}

/// The wordmark as it stands at `elapsed`: a line a row, grain and tint as the
/// wave leaves them. A tint depends only on where a cell is, never on what it
/// draws, so a row comes out as a handful of spans rather than one a cell.
pub fn lines(rows: &[&str], theme: &Theme, elapsed: Duration) -> Vec<Line<'static>> {
    let (sweep, tints) = (Sweep::over(rows), tints(theme));
    rows.iter().enumerate().map(|(row, text)| line(row, text, rows, &sweep, &tints, elapsed)).collect()
}

fn line(row: usize, text: &str, rows: &[&str], sweep: &Sweep, tints: &[Color; TINTS], elapsed: Duration) -> Line<'static> {
    let mut spans: Vec<Span<'static>> = Vec::new();
    let mut run = String::new();
    let mut colour: Option<Color> = None;

    for (column, glyph) in text.chars().enumerate() {
        let band = sweep.band(row, column, elapsed);
        let cell = tone(row, band, rows, tints);
        if let Some(previous) = colour
            && previous != cell
        {
            spans.push(Span::styled(std::mem::take(&mut run), Style::default().fg(previous)));
        }
        colour = Some(cell);
        run.push(grain(glyph, row, column, stir(band), elapsed));
    }

    if let Some(colour) = colour {
        spans.push(Span::styled(run, Style::default().fg(colour)));
    }
    Line::from(spans)
}

#[cfg(test)]
#[path = "../tests/helpers/logo.rs"]
mod tests;
