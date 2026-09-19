use ratatui::{Frame, layout::Rect};
use tui_term::widget::PseudoTerminal;

use crate::core::pty::PtySession;

/// Draws the focused agent's terminal, and puts the real cursor where the agent
/// thinks it is so typing looks normal.
pub fn draw(frame: &mut Frame, area: Rect, session: &PtySession) {
    let Ok(parser) = session.parser().lock() else {
        return;
    };
    let screen = parser.screen();
    frame.render_widget(PseudoTerminal::new(screen), area);

    if !screen.hide_cursor() {
        let (row, col) = screen.cursor_position();
        frame.set_cursor_position((area.x + col, area.y + row));
    }
}
