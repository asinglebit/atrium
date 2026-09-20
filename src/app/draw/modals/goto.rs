use ratatui::{
    Frame,
    layout::Rect,
    style::Style,
    text::{Line, Span},
    widgets::{Clear, List, Paragraph},
};

use crate::{
    app::{
        draw::pane,
        state::{goto::Goto, layout},
    },
    core::registry::Registry,
    helpers::palette::Theme,
};

const WIDTH: u16 = 50;
/// Two lines of chrome plus a row each, capped so a long list still fits.
const MAX_ROWS: u16 = 12;

/// The "which one" box: every held agent, jumpable by its own number.
pub fn draw(frame: &mut Frame, full: Rect, goto: &Goto, registry: &Registry, theme: &Theme, spinner: char, lit: bool) {
    let rows = (registry.len() as u16).min(MAX_ROWS);
    let area = layout::centered(WIDTH, rows + 3, full);
    // Without this the stage shows through the gaps in the modal.
    frame.render_widget(Clear, area);

    let block = pane::modal_block(theme, " go to ");
    let inner = block.inner(area);
    frame.render_widget(block, area);

    let [list_area, footer] = layout::stack_footer(inner, 1);

    let lines = pane::agent_lines(registry, theme, spinner, lit, list_area.width as usize);
    let items = pane::zebra_list_items(lines, list_area.height as usize, goto.selected(), true, theme);
    frame.render_widget(List::new(items), list_area);

    frame.render_widget(Paragraph::new(Line::from(Span::styled("1-0 jump · j/k move · enter pick · esc cancel", Style::default().fg(theme.COLOR_GREY_600)))), footer);
}

#[cfg(test)]
#[path = "../../../tests/app/draw/modals/goto.rs"]
mod tests;
