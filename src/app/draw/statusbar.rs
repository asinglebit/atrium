use ratatui::{
    Frame,
    style::Style,
    text::{Line, Span},
    widgets::Paragraph,
};

use crate::{app::state::layout::Layout, core::registry::Registry, helpers::palette::Theme};

/// The line under the frame: which agent is showing and where it sits in the
/// list, in the same shape guitar uses.
pub fn draw(frame: &mut Frame, layout: &Layout, registry: &Registry, theme: &Theme) {
    let mut left = vec![Span::raw("  ")];
    if let Some(agent) = registry.focused() {
        left.push(Span::styled(format!("{} ", agent.name), Style::default().fg(theme.COLOR_TEXT)));
        left.push(Span::styled(agent.status.label(), Style::default().fg(crate::app::draw::pane::status_color(theme, agent.status))));
        if let Some(git) = agent.git() {
            let dirty = if git.dirty { "*" } else { "" };
            left.push(Span::styled(format!("  ● {}{dirty}", git.branch), Style::default().fg(theme.COLOR_GRASS)));
        }
    }
    frame.render_widget(Paragraph::new(Line::from(left)).left_aligned(), layout.statusbar_left);

    let position = if registry.is_empty() { String::new() } else { format!("{}/{} ", registry.focus() + 1, registry.len()) };
    frame.render_widget(Paragraph::new(Line::from(Span::styled(position, Style::default().fg(theme.COLOR_TEXT)))).right_aligned(), layout.statusbar_right);
}
