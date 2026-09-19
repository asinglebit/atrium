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

#[cfg(test)]
#[path = "../../tests/app/state/layout.rs"]
mod tests;
