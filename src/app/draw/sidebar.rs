use ratatui::{
    Frame,
    layout::Rect,
    widgets::{List, ListItem},
};

use crate::{app::draw::pane, core::registry::Registry, helpers::palette::Theme};

/// Draws the held agents, striped and highlighted the way guitar's panes are.
pub fn draw(frame: &mut Frame, area: Rect, registry: &Registry, theme: &Theme, spinner: char, offset: usize) {
    let split = area.width.saturating_sub(1);
    let body = Rect { width: split, ..area };
    let gutter = Rect { x: area.x + split, width: 1, ..area };

    let block = pane::block(theme, format!(" agents {} ", registry.len()));
    let inner = block.inner(body);
    frame.render_widget(block, body);

    let visible = inner.height as usize;
    let lines = pane::agent_lines(registry, theme, spinner, inner.width as usize);
    let shown: Vec<_> = lines.into_iter().skip(offset).collect();
    let selected = registry.focus().saturating_sub(offset);

    let items: Vec<ListItem> = pane::zebra_list_items(shown, visible, selected, true, theme);
    frame.render_widget(List::new(items), inner);

    pane::draw_gutter(frame, gutter, theme, registry.len(), visible, offset);
}

/// Which agent is under a click. Lives here so it cannot drift from the layout
/// the draw above actually used: one title line, then a row per agent.
pub fn row_at(area: Rect, offset: usize, row: u16) -> Option<usize> {
    let first = area.y.checked_add(1)?;
    (row >= first && row < area.y.saturating_add(area.height)).then(|| offset + usize::from(row - first))
}

#[cfg(test)]
#[path = "../../tests/app/draw/sidebar.rs"]
mod tests;
