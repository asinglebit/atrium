use ratatui::{Frame, layout::Rect};
use tui_term::widget::PseudoTerminal;

use crate::{core::pty::PtySession, helpers::palette::Theme};

/// Draws the focused agent's terminal, and puts the real cursor where the agent
/// thinks it is so typing looks normal.
pub fn draw(frame: &mut Frame, area: Rect, session: &PtySession, theme: &Theme) {
    let Ok(parser) = session.parser().lock() else {
        return;
    };
    let screen = parser.screen();
    frame.render_widget(PseudoTerminal::new(screen), area);

    // After the widget, never before: it clears the area first, so anything
    // painted underneath is gone by the time it returns. `PseudoTerminal::style`
    // looks like it would do this, but tui-term never reads it.
    theme.fill_default_background(area, frame.buffer_mut());

    if !screen.hide_cursor() {
        let (row, col) = screen.cursor_position();
        frame.set_cursor_position((area.x + col, area.y + row));
    }
}

#[cfg(test)]
#[path = "../../tests/app/draw/stage.rs"]
mod tests;
