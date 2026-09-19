use ratatui::{
    Frame,
    layout::Rect,
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, ListState},
};

use crate::{
    core::{agent::Status, registry::Registry},
    helpers::palette::Theme,
};

/// Draws the held agents, one row each, with the focused row highlighted.
pub fn draw(frame: &mut Frame, area: Rect, registry: &Registry, theme: &Theme, spinner: char) {
    let items: Vec<ListItem> = registry
        .agents()
        .iter()
        .enumerate()
        .map(|(index, agent)| {
            // Only the first nine get a jump key, so only they are numbered.
            let key = if index < 9 { format!("{} ", index + 1) } else { "  ".to_owned() };
            let style = Style::default().fg(if agent.has_exited() { theme.exited } else { theme.text });
            // A working agent spins where the others show a steady glyph.
            let mark = if agent.status == Status::Working { spinner.to_string() } else { agent.status.glyph().to_owned() };
            let mut lines = vec![Line::from(vec![Span::styled(format!(" {mark} "), Style::default().fg(theme.for_status(agent.status))), Span::raw(key), Span::raw(agent.name.clone())])];
            if let Some(git) = agent.git() {
                // An asterisk is the whole dirty/clean signal; a count would not fit.
                let dirty = if git.dirty { " *" } else { "" };
                lines.push(Line::from(Span::styled(format!("     {}{}", git.branch, dirty), Style::default().fg(theme.dim))));
            }
            ListItem::new(lines).style(style)
        })
        .collect();

    let block = Block::default().borders(Borders::RIGHT).border_style(Style::default().fg(theme.border)).title(format!(" agents {} ", registry.len()));
    let list = List::new(items).block(block).highlight_style(Style::default().add_modifier(Modifier::REVERSED));

    let mut state = ListState::default();
    state.select(Some(registry.focus()));
    frame.render_stateful_widget(list, area, &mut state);
}

#[cfg(test)]
#[path = "../../tests/app/draw/sidebar.rs"]
mod tests;
