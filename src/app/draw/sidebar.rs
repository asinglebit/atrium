use ratatui::{
    Frame,
    layout::Rect,
    widgets::{List, ListItem},
};

use crate::{app::draw::pane, core::registry::Registry, helpers::palette::Theme};

/// Draws the held agents, one row each, striped and highlighted the way
/// guitar's panes are.
pub fn draw(frame: &mut Frame, area: Rect, registry: &Registry, theme: &Theme, spinner: char) {
    let block = pane::block(theme, format!(" agents {} ", registry.len()));
    let inner = block.inner(area);
    frame.render_widget(block, area);

    let lines = pane::agent_lines(registry, theme, spinner, inner.width as usize);

    let items: Vec<ListItem> = pane::zebra_list_items(lines, inner.height as usize, registry.focus(), true, theme);
    frame.render_widget(List::new(items), inner);
}

#[cfg(test)]
#[path = "../../tests/app/draw/sidebar.rs"]
mod tests;
