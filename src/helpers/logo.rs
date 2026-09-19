use ratatui::style::Color;

use crate::helpers::palette::Theme;

/// atrium's wordmark at its largest, for the splash. guitar's own splash logo
/// is the same shape and height.
pub const WIDE: [&str; 11] = [
    "                                                      ",
    "                        68b                           ",
    "          /             Y89                           ",
    "   ___   /M     ___  __ ___ ___   ___ ___  __    __   ",
    " 6MMMMb /MMMMM  `MM 6MM `MM `MM    MM `MM 6MMb  6MMb  ",
    "8M'  `Mb MM      MM69 \"  MM  MM    MM  MM69 `MM69 `Mb ",
    "    ,oMM MM      MM'     MM  MM    MM  MM'   MM'   MM ",
    ",6MM9'MM MM      MM      MM  MM    MM  MM    MM    MM ",
    "MM'   MM MM      MM      MM  MM    MM  MM    MM    MM ",
    "MM.  ,MM YM.  ,  MM      MM  YM.   MM  MM    MM    MM ",
    "`YMMM9'Yb.YMMM9 _MM_    _MM_  YMMM9MM__MM_  _MM_  _MM_",
];

pub const WIDE_WIDTH: usize = 54;

/// The same word in half the height, for a header with less room to give than
/// the splash -- which is the settings view.
///
/// The top two rows carry only the dot on the `i` and the ascender on the `t`;
/// the three below are the x-height every letter shares.
pub const BLOCK: [&str; 5] = ["      █      █           ", "      █                  ", "▄▀▀█ ███ █▄▄ █ █  █ █▀█▀█", "█  █  █  █   █ █  █ █ █ █", "▀▀▀▀  ▀▀ █   █  ▀▀▀ █ █ █"];

pub const BLOCK_WIDTH: usize = 25;

/// What a column with room for neither gets instead.
pub const COMPACT: &str = "atrium";

/// How many of `total` rows take the lighter tone: the top 30%, rounded up so a
/// short wordmark still gets some, and never fewer than one.
pub fn bright_rows(total: usize) -> usize {
    if total == 0 { 0 } else { (total * 3).div_ceil(10).max(1) }
}

/// What row `index` is drawn in. Both tones come from the palette, so a retheme
/// carries the wordmark with it, and living here is what keeps the splash and
/// the settings header from drifting apart.
///
/// The share is counted over the rows that carry ink, not over all of them: the
/// wide wordmark opens with a blank row, and counting it would spend a third of
/// the lighter tone on a row that paints nothing.
pub fn tone(index: usize, rows: &[&str], theme: &Theme) -> Color {
    let first = rows.iter().position(|row| !row.trim().is_empty()).unwrap_or(0);
    if index < first + bright_rows(rows.len() - first) { theme.COLOR_PINK } else { theme.COLOR_PURPLE }
}

/// The block or the word. What the settings header uses, which cannot afford
/// eleven rows before the first setting.
pub fn rows_for(width: usize) -> &'static [&'static str] {
    if width >= BLOCK_WIDTH { &BLOCK } else { std::slice::from_ref(&COMPACT) }
}

/// The biggest that fits. What the splash uses, where the wordmark is the point.
pub fn splash_rows_for(width: usize) -> &'static [&'static str] {
    if width >= WIDE_WIDTH { &WIDE } else { rows_for(width) }
}

#[cfg(test)]
#[path = "../tests/helpers/logo.rs"]
mod tests;
