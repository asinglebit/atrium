use ratatui::layout::Rect;

/// How wide the sidebar is when there is room for it.
pub const SIDEBAR_WIDTH: u16 = 28;

/// Below this the sidebar is dropped entirely rather than squeezing the agent
/// into a column too narrow to work in.
pub const MIN_STAGE_WIDTH: u16 = 40;

/// Where everything goes. Same shape as guitar's: a title line, a bordered
/// frame holding the panes, and a status line under it.
#[derive(Clone, Copy, Debug)]
pub struct Layout {
    pub title_left: Rect,
    pub title_right: Rect,
    pub app: Rect,
    /// None on a terminal too narrow to afford one.
    pub sidebar: Option<Rect>,
    pub stage: Rect,
    pub statusbar_left: Rect,
    pub statusbar_right: Rect,
}

/// How much of the title and status lines the left half gets.
const LEFT_SHARE: u16 = 70;

fn halves(row: Rect) -> (Rect, Rect) {
    let left = row.width * LEFT_SHARE / 100;
    (Rect { width: left, ..row }, Rect { x: row.x + left, width: row.width - left, ..row })
}

pub fn compute(full: Rect, want_sidebar: bool) -> Layout {
    // A title line above and a status line below, the frame taking the rest.
    let title = Rect { height: 1.min(full.height), ..full };
    let status_height = if full.height >= 3 { 1 } else { 0 };
    let app = Rect { y: full.y + title.height, height: full.height.saturating_sub(title.height + status_height), ..full };
    let statusbar = Rect { y: app.y + app.height, height: status_height, ..full };

    // The frame's own border is not floor space.
    let inner = Rect { x: app.x + 1, y: app.y + 1, width: app.width.saturating_sub(2), height: app.height.saturating_sub(2) };

    let (sidebar, stage) = if !want_sidebar || inner.width < SIDEBAR_WIDTH + MIN_STAGE_WIDTH {
        (None, inner)
    } else {
        let sidebar = Rect { width: SIDEBAR_WIDTH, ..inner };
        let stage = Rect { x: inner.x + SIDEBAR_WIDTH, width: inner.width - SIDEBAR_WIDTH, ..inner };
        (Some(sidebar), stage)
    };

    let (title_left, title_right) = halves(title);
    let (statusbar_left, statusbar_right) = halves(statusbar);

    Layout { title_left, title_right, app, sidebar, stage, statusbar_left, statusbar_right }
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

/// Splits a box into whatever is above a fixed-height footer, and the footer.
pub fn stack_footer(area: Rect, footer_height: u16) -> [Rect; 2] {
    let footer_height = footer_height.min(area.height);
    let body = Rect { height: area.height - footer_height, ..area };
    let footer = Rect { y: area.y + body.height, height: footer_height, ..area };
    [body, footer]
}

#[cfg(test)]
#[path = "../../tests/app/state/layout.rs"]
mod tests;
