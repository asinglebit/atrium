use ratatui::{
    Frame,
    layout::Rect,
    style::Style,
    text::{Line, Span},
    widgets::{List, ListItem},
};

use crate::{
    app::draw::pane,
    core::{agent::Status, registry::Registry},
    helpers::{palette::Theme, text::truncate},
};

/// Draws the held agents, one row each, striped and highlighted the way
/// guitar's panes are.
pub fn draw(frame: &mut Frame, area: Rect, registry: &Registry, theme: &Theme, spinner: char) {
    let block = pane::block(theme, format!(" agents {} ", registry.len()));
    let inner = block.inner(area);
    frame.render_widget(block, area);

    let width = inner.width as usize;
    let branches: Vec<String> = registry.agents().iter().map(|agent| agent.git().map(|git| format!("{}{}", git.branch, if git.dirty { "*" } else { "" })).unwrap_or_default()).collect();

    // One branch column for every row, so the names line up on the left and the
    // branches line up on the right instead of each row finding its own edge.
    let marker = 4;
    let room = width.saturating_sub(marker);
    let branch_width = branches.iter().map(|branch| branch.chars().count()).max().unwrap_or(0).min(room.saturating_sub(MIN_NAME_WIDTH + 1));
    let name_width = room.saturating_sub(if branch_width == 0 { 0 } else { branch_width + 1 });

    let lines: Vec<Line> = registry
        .agents()
        .iter()
        .zip(&branches)
        .enumerate()
        .map(|(index, (agent, branch))| {
            // Only the first nine get a jump key, so only they are numbered.
            let key = if index < 9 { format!("{} ", index + 1) } else { "  ".to_owned() };
            // A working agent spins where the others show a steady glyph.
            let mark = if agent.status == Status::Working { spinner.to_string() } else { agent.status.glyph().to_owned() };
            let body = if agent.has_exited() { theme.COLOR_GREY_600 } else { theme.COLOR_GREY_300 };

            let mut spans = vec![
                Span::styled(format!("{mark} "), Style::default().fg(pane::status_color(theme, agent.status))),
                Span::styled(key, Style::default().fg(theme.COLOR_GREY_600)),
                Span::styled(format!("{:<name_width$}", truncate(&agent.name, name_width)), Style::default().fg(body)),
            ];
            if branch_width > 0 {
                spans.push(Span::styled(format!(" {:>branch_width$}", truncate(branch, branch_width)), Style::default().fg(theme.COLOR_GREY_600)));
            }
            Line::from(spans)
        })
        .collect();

    let items: Vec<ListItem> = pane::zebra_list_items(lines, inner.height as usize, registry.focus(), true, theme);
    frame.render_widget(List::new(items), inner);
}

/// However long the branches are, a name never gets squeezed below this.
const MIN_NAME_WIDTH: usize = 8;

#[cfg(test)]
#[path = "../../tests/app/draw/sidebar.rs"]
mod tests;
