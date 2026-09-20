use ratatui::style::Color;

use crate::helpers::palette::Theme;

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

/// The word in five rows, for a header with less room to give than the splash
/// -- which is the settings view.
///
/// The top two rows carry only the dot on the `i` and the ascender on the `t`;
/// the three below are the x-height every letter shares.
pub const BLOCK: [&str; 5] = ["      █      █           ", "      █                  ", "▄▀▀█ ███ █▄▄ █ █  █ █▀█▀█", "█  █  █  █   █ █  █ █ █ █", "▀▀▀▀  ▀▀ █   █  ▀▀▀ █ █ █"];

pub const BLOCK_WIDTH: usize = 25;

/// What a column with room for none of them gets instead. The hooked `ɱ` is the
/// drawn wordmark's tail kept at one row -- guitar ends its own compact logo on
/// a corner glyph the same way.
pub const COMPACT: &str = "atriuɱ";

/// Where the splash changes wordmark. Guitar's two breakpoints, not the widths
/// the art happens to need: a wordmark reaching the edges of the terminal is
/// not the same picture as one with room left around it.
const WIDE_COLUMNS: usize = 120;
const NARROW_COLUMNS: usize = 80;

/// How many of `total` rows take the lighter tone: the top 30%, rounded up so a
/// short wordmark still gets some, and never fewer than one.
pub fn bright_rows(total: usize) -> usize {
    if total == 0 { 0 } else { (total * 3).div_ceil(10).max(1) }
}

/// What row `index` is drawn in. Both tones come from the palette, so a retheme
/// carries the wordmark with it, and living here is what keeps the splash and
/// the settings header from drifting apart.
///
/// Every wordmark opens on a row that carries ink -- the dot of the `i` at the
/// least -- so a plain share of the rows puts the lighter tone on the tops of
/// the letters, which is where it reads.
pub fn tone(index: usize, rows: &[&str], theme: &Theme) -> Color {
    if index < bright_rows(rows.len()) { theme.COLOR_PINK } else { theme.COLOR_PURPLE }
}

/// The block or the word. What the settings header uses, which cannot afford a
/// drawn wordmark before the first setting.
pub fn rows_for(width: usize) -> &'static [&'static str] {
    if width >= BLOCK_WIDTH { &BLOCK } else { std::slice::from_ref(&COMPACT) }
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

#[cfg(test)]
#[path = "../tests/helpers/logo.rs"]
mod tests;
