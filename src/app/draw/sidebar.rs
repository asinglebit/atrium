use ratatui::{
    Frame,
    layout::Rect,
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, ListState},
};

use crate::core::registry::Registry;

/// Draws the held agents, one row each, with the focused row highlighted.
pub fn draw(frame: &mut Frame, area: Rect, registry: &Registry) {
    let items: Vec<ListItem> = registry
        .agents()
        .iter()
        .enumerate()
        .map(|(index, agent)| {
            // Only the first nine get a jump key, so only they are numbered.
            let key = if index < 9 { format!("{} ", index + 1) } else { "  ".to_owned() };
            let style = if agent.has_exited() { Style::default().add_modifier(Modifier::DIM) } else { Style::default() };
            ListItem::new(Line::from(vec![Span::raw(format!(" {} ", agent.status.glyph())), Span::raw(key), Span::raw(agent.name.clone())])).style(style)
        })
        .collect();

    let block = Block::default().borders(Borders::RIGHT).title(format!(" agents {} ", registry.len()));
    let list = List::new(items).block(block).highlight_style(Style::default().add_modifier(Modifier::REVERSED));

    let mut state = ListState::default();
    state.select(Some(registry.focus()));
    frame.render_stateful_widget(list, area, &mut state);
}

#[cfg(test)]
#[path = "../../tests/app/draw/sidebar.rs"]
mod tests;
