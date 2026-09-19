use ratatui::{
    Frame,
    style::Style,
    text::{Line, Span},
    widgets::Paragraph,
};

use crate::{app::state::layout::Layout, helpers::palette::Theme, helpers::text::truncate_start};

/// The same glyph guitar puts in front of a path.
const FOLDER: &str = "";

/// The line above the frame: what this is and where it is on the left, what you
/// are looking at on the right.
pub fn draw(frame: &mut Frame, layout: &Layout, theme: &Theme, cwd: &str, view: &str) {
    let room = layout.title_left.width.saturating_sub(15) as usize;

    let left = Line::from(vec![
        // atrium's own colour, the same purple the logo is drawn in.
        Span::styled("  atrium", Style::default().fg(theme.COLOR_PURPLE)),
        Span::styled(" |", Style::default().fg(theme.COLOR_TEXT)),
        Span::styled(format!(" {FOLDER} {}", truncate_start(cwd, room)), Style::default().fg(theme.COLOR_TEXT)),
    ]);
    frame.render_widget(Paragraph::new(left).left_aligned(), layout.title_left);

    let right = Line::from(Span::styled(format!("{view} "), Style::default().fg(theme.COLOR_HIGHLIGHTED)));
    frame.render_widget(Paragraph::new(right).right_aligned(), layout.title_right);
}
