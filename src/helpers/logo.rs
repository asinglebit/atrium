/// atrium's wordmark, in the shape guitar's splash uses: a block of rows above
/// the content, drawn in the tool's own colour.
///
/// The top two rows carry only the dot on the `i` and the ascender on the `t`;
/// the three below are the x-height every letter shares.
pub const WIDE: [&str; 5] = ["      █      █           ", "      █                  ", "▄▀▀█ ███ █▄▄ █ █  █ █▀█▀█", "█  █  █  █   █ █  █ █ █ █", "▀▀▀▀  ▀▀ █   █  ▀▀▀ █ █ █"];

/// How wide the block above is, which is the narrowest column that can hold it.
pub const WIDTH: usize = 25;

/// How many of the rows are drawn in the brighter of the two purples. guitar
/// splits its own logo across two greens at the same place.
pub const BRIGHT_ROWS: usize = 2;

/// What a frame with no room for the block gets instead.
pub const COMPACT: &str = "atrium";

/// The rows to draw in a column this wide, and whether they are the block.
pub fn rows_for(width: usize) -> &'static [&'static str] {
    if width >= WIDTH { &WIDE } else { std::slice::from_ref(&COMPACT) }
}

#[cfg(test)]
#[path = "../tests/helpers/logo.rs"]
mod tests;
