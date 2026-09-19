use ratatui::layout::Rect;

/// How wide the sidebar is when there is room for it.
pub const SIDEBAR_WIDTH: u16 = 26;

/// Below this the sidebar is dropped entirely rather than squeezing the agent
/// into a column too narrow to work in.
pub const MIN_STAGE_WIDTH: u16 = 40;

/// Splits the frame into a sidebar and the stage the focused agent draws on.
/// The sidebar is absent on a terminal too narrow to afford one.
pub fn split(full: Rect) -> (Option<Rect>, Rect) {
    if full.width < SIDEBAR_WIDTH + MIN_STAGE_WIDTH {
        return (None, full);
    }
    let sidebar = Rect { width: SIDEBAR_WIDTH, ..full };
    let stage = Rect { x: full.x + SIDEBAR_WIDTH, width: full.width - SIDEBAR_WIDTH, ..full };
    (Some(sidebar), stage)
}

/// A box of the given size in the middle of `full`, shrunk to fit if it does
/// not have room.
pub fn centered(width: u16, height: u16, full: Rect) -> Rect {
    let width = width.min(full.width);
    let height = height.min(full.height);
    Rect { x: full.x + (full.width - width) / 2, y: full.y + (full.height - height) / 2, width, height }
}

/// Splits a box into a fixed-height header and whatever is left below it.
pub fn stack_header(area: Rect, header_height: u16) -> [Rect; 2] {
    let header_height = header_height.min(area.height);
    let header = Rect { height: header_height, ..area };
    let rest = Rect { y: area.y + header_height, height: area.height - header_height, ..area };
    [header, rest]
}

#[cfg(test)]
#[path = "../../tests/app/state/layout.rs"]
mod tests;
